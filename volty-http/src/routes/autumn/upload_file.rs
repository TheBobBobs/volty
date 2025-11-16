use reqwest::multipart::{Form, Part};
use serde::{Deserialize, Serialize};

use crate::{bucket::BucketKey, error::HttpError, Http};

#[derive(Clone, Copy, Debug)]
pub enum Tag {
    Attachments,
    Avatars,
    Backgrounds,
    Icons,
    Banners,
    Emojis,
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Tag::Attachments => "attachments",
            Tag::Avatars => "avatars",
            Tag::Backgrounds => "backgrounds",
            Tag::Icons => "icons",
            Tag::Banners => "banners",
            Tag::Emojis => "emojis",
        };
        f.write_str(text)
    }
}

#[derive(Debug)]
pub struct UploadFile {
    form: Form,
}

impl UploadFile {
    pub fn new(bytes: Vec<u8>, file_name: Option<impl std::fmt::Display>) -> Self {
        let mut part = Part::bytes(bytes);
        if let Some(file_name) = file_name {
            part = part.file_name(file_name.to_string());
        }
        Self {
            form: Form::new().part("file", part),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UploadResponse {
    pub id: String,
}

impl Http {
    pub async fn upload_file(
        &self,
        tag: Tag,
        file: UploadFile,
    ) -> Result<UploadResponse, HttpError> {
        let url = format!("https://cdn.revoltusercontent.com/{tag}");
        let request = self.client.post(url).multipart(file.form);
        let response = request.send().await;
        self.handle_response(response, BucketKey::Channels("autumn".to_string()))
            .await
    }
}
