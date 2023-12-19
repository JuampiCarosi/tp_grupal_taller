use std::{collections::HashMap, sync::Arc};

use crate::{servidor::repo_storage::RepoStorage, tipos_de_dato::logger::Logger};

use super::{error::ErrorHttp, request::Request, response::Response};

pub type EndpointHandler =
    fn(Request, HashMap<String, String>, Arc<Logger>, RepoStorage) -> Result<Response, ErrorHttp>;
