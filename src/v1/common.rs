use std::fmt;

#[allow(clippy::trivially_copy_pass_by_ref)]
fn pretty_on(pretty_print: &bool) -> bool {
    *pretty_print
}

/// [Standard Query Parameters](https://cloud.google.com/storage/docs/json_api/v1/parameters#query)
/// can be used in almost any API request to GCS
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StandardQueryParameters<'a> {
    /// Selector specifying a subset of fields to include in the response,
    /// the primary use of this is for better performance and lower response
    /// sizes.
    /// For more information, see the [partial response](https://cloud.google.com/storage/docs/json_api/v1/how-tos/performance#partial)
    /// documentation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<&'a str>,
    /// Returns the response in a human-readable format, with indentations and
    /// line breaks, if true. Note that while the default value is `true` for
    /// GCP, this crate uses a default of `false`
    #[serde(skip_serializing_if = "pretty_on")]
    pub pretty_print: bool,
    /// Lets you enforce per-user quotas from a server-side application even
    /// in cases when the user's IP address is unknown. This can occur, for
    /// example, with applications that run cron jobs on App Engine on a
    /// user's behalf. You can choose any arbitrary string that uniquely
    /// identifies a user, but it is limited to 40 characters. Overrides
    /// `userIp` if both are provided. See more about [Capping API usage](https://cloud.google.com/apis/docs/capping-api-usage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_user: Option<&'a str>,
    /// Lets you enforce per-user quotas when calling the API from a server-side application.
    /// See more about [Capping API usage](https://cloud.google.com/apis/docs/capping-api-usage)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_ip: Option<&'a str>,
}

impl<'a> Default for StandardQueryParameters<'a> {
    fn default() -> Self {
        Self {
            fields: None,
            pretty_print: false,
            quota_user: None,
            user_ip: None,
        }
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Conditionals {
    /// Makes the operation conditional on whether the object's current
    /// generation matches the given value. Setting to 0 makes the
    /// operation succeed only if there are no live versions of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_generation_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// generation does not match the given value. If no live object exists,
    /// the precondition fails. Setting to 0 makes the operation succeed only
    /// if there is a live version of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_generation_not_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration matches the given value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_metageneration_match: Option<i64>,
    /// Makes the operation conditional on whether the object's current
    /// metageneration does not match the given value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub if_metageneration_not_match: Option<i64>,
}

/// [Storage classes](https://cloud.google.com/storage/docs/storage-classes)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum StorageClass {
    /// [Multi-Regional Storage](https://cloud.google.com/storage/docs/storage-classes#multi-regional)
    /// is appropriate for storing data that is frequently accessed ("hot" objects), such as serving
    /// website content, interactive workloads, or data supporting mobile and gaming applications.
    /// Multi-Regional Storage data has the most availability compared to other storage classes.
    MultiRegional,
    /// [Regional Storage](https://cloud.google.com/storage/docs/storage-classes#regional) enables
    /// you to store data at lower cost, with the trade-off of data being stored in a specific
    /// regional location, instead of having redundancy distributed over a large geographic area.
    Regional,
    /// [Nearline Storage](https://cloud.google.com/storage/docs/storage-classes#nearline) is a
    /// low-cost, highly durable storage service for storing infrequently accessed data.
    /// Nearline Storage is a better choice than Multi-Regional Storage or Regional Storage
    /// in scenarios where slightly lower availability, a 30-day minimum storage duration,
    /// and costs for data access are acceptable trade-offs for lowered storage costs.
    Nearline,
    /// [Coldline Storage](https://cloud.google.com/storage/docs/storage-classes#coldline)
    /// is a very-low-cost, highly durable storage service for data archiving, online backup,
    /// and disaster recovery. Unlike other "cold" storage services, your data is available
    /// within milliseconds, not hours or days.
    Coldline,
    /// Users that create a bucket without specifying a default storage class see the bucket's
    /// default storage class listed as [Standard Storage](https://cloud.google.com/storage/docs/storage-classes#standard)
    /// in the API. Objects created without a storage class in such a bucket are also listed
    /// as Standard Storage in the API. Standard Storage is equivalent to Multi-Regional
    /// Storage when the associated bucket is located in a multi-regional location. Standard
    /// Storage is equivalent to Regional Storage when the associated bucket is located in a
    /// regional location.
    Standard,
    /// It is recommended that users utilize Regional Storage in place of [Durable Reduced Availability (DRA)](https://cloud.google.com/storage/docs/storage-classes#dra).
    /// Regional Storage has lower pricing for operations, but otherwise the same price structure.
    /// Regional Storage also has better performance, particularly in terms of availability
    /// (DRA has a 99% availability SLA).
    DurableReducedAvailability,
}

impl fmt::Display for StorageClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PredefinedAcl {
    /// Object owner gets OWNER access, and allAuthenticatedUsers get READER access.
    AuthenticatedRead,
    /// Object owner gets OWNER access, and project team owners get OWNER access.
    BucketOwnerFullControl,
    /// Object owner gets OWNER access, and project team owners get READER access.
    BucketOwnerRead,
    /// Object owner gets OWNER access.
    Private,
    /// Object owner gets OWNER access, and project team members get access according to their roles.
    ProjectPrivate,
    /// Object owner gets OWNER access, and allUsers get READER access.
    PublicRead,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Projection {
    /// Include all properties.
    Full,
    /// Omit the owner, acl property.
    NoAcl,
}
