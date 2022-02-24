use super::*;
use futures_util::{
    io::{AsyncRead, Result as FuturesResult},
    task::{Context, Poll},
    Stream,
};
use std::pin::Pin;

impl<B: AsyncRead + Unpin> AsyncRead for Multipart<B> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<FuturesResult<usize>> {
        use std::cmp::min;
        let mut total_copied = 0;

        let (copied, len) = match self.cursor.part {
            MultipartPart::Prefix => {
                let to_copy = min(buf.len(), self.prefix.len() - self.cursor.position);

                buf[..to_copy].copy_from_slice(
                    &self.prefix[self.cursor.position..self.cursor.position + to_copy],
                );

                (to_copy, self.prefix.len())
            }
            MultipartPart::Body => {
                let copied = match self.as_mut().body().poll_read(cx, buf) {
                    Poll::Ready(Ok(copied)) => copied,
                    other => return other,
                };
                (copied, self.body_len as usize)
            }
            MultipartPart::Suffix => {
                let to_copy = min(buf.len(), MULTI_PART_SUFFIX.len() - self.cursor.position);

                buf[..to_copy].copy_from_slice(
                    &MULTI_PART_SUFFIX[self.cursor.position..self.cursor.position + to_copy],
                );

                (to_copy, MULTI_PART_SUFFIX.len())
            }
            MultipartPart::End => return Poll::Ready(Ok(0)),
        };

        self.cursor.position += copied;
        total_copied += copied;

        if self.cursor.position == len {
            self.cursor.part.next();
            self.cursor.position = 0;
        }

        Poll::Ready(Ok(total_copied))
    }
}

impl Stream for Multipart<bytes::Bytes> {
    type Item = bytes::Bytes;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(match self.cursor.part {
            MultipartPart::Prefix => {
                self.cursor.part.next();
                Some(self.prefix.clone())
            }
            MultipartPart::Body => {
                self.cursor.part.next();
                Some(self.body.clone())
            }
            MultipartPart::Suffix => {
                self.cursor.part.next();
                Some(bytes::Bytes::from(MULTI_PART_SUFFIX))
            }
            MultipartPart::End => None,
        })
    }
}
