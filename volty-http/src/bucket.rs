use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use reqwest::{Method, Response};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum BucketKey {
    Auth,
    AuthDelete,
    Bots,
    Channels(String),
    DefaultAvatar,
    Messaging(String),
    Safety,
    SafetyReport,
    Servers(String),
    Swagger,
    Users,
    UserEdit(String),
    Any,
}

impl BucketKey {
    pub fn new(method: Method, path: &str) -> Self {
        let mut segments = path.split('/');
        let segment_0 = segments.next().unwrap_or("");
        let segment_1 = segments.next().unwrap_or("");
        let segment_2 = segments.next().unwrap_or("");
        use BucketKey::*;
        match (method, segment_0, segment_1, segment_2) {
            (Method::PATCH, "users", target, ..) => UserEdit(target.into()),
            (_, "users", _, "default_avatar") => DefaultAvatar,
            (_, "users", ..) => Users,
            (_, "bots", ..) => Bots,

            (Method::POST, "channels", target, "messages") => Messaging(target.into()),
            (_, "channels", target, ..) => Channels(target.into()),

            (_, "servers", target, ..) => Servers(target.into()),

            (Method::DELETE, "auth", ..) => AuthDelete,
            (_, "auth", ..) => Auth,

            (_, "swagger", ..) => Swagger,

            (_, "safety", "report", ..) => SafetyReport,
            (_, "safety", ..) => Safety,
            _ => Any,
        }
    }

    fn limit(&self) -> u8 {
        match self {
            BucketKey::Auth => 15,
            BucketKey::AuthDelete => 255,
            BucketKey::Bots => 10,
            BucketKey::Channels(_) => 15,
            BucketKey::DefaultAvatar => 255,
            BucketKey::Messaging(_) => 10,
            BucketKey::Safety => 15,
            BucketKey::SafetyReport => 3,
            BucketKey::Servers(_) => 5,
            BucketKey::Swagger => 100,
            BucketKey::Users => 20,
            BucketKey::UserEdit(_) => 2,
            BucketKey::Any => 20,
        }
    }
}

#[derive(Debug)]
struct Bucket {
    used: u8,
    reset: Instant,
}

impl Bucket {
    fn deduct(&mut self, limit: u8) -> Result<(), Duration> {
        if self.remaining(limit) > 0 {
            self.used += 1;
            Ok(())
        } else {
            Err(self.reset - Instant::now())
        }
    }

    fn remaining(&mut self, limit: u8) -> u8 {
        let now = Instant::now();
        if now >= self.reset {
            self.used = 0;
            self.reset = now + Duration::from_secs(10);
        }
        limit - self.used.min(limit)
    }
}

pub struct Buckets {
    buckets: Mutex<HashMap<BucketKey, Bucket>>,
}

impl Buckets {
    pub fn new() -> Self {
        Self {
            buckets: Mutex::new(HashMap::new()),
        }
    }

    pub fn take(&self, key: &BucketKey) -> Result<(), Duration> {
        let mut buckets = self.buckets.lock().unwrap();
        if let Some(bucket) = buckets.get_mut(key) {
            bucket.deduct(key.limit())
        } else {
            let bucket = Bucket {
                used: 1,
                reset: Instant::now() + Duration::from_secs(10),
            };
            buckets.insert(key.clone(), bucket);
            Ok(())
        }
    }

    pub fn handle_response(&self, key: &BucketKey, response: &Response) {
        let headers = response.headers();
        let Some(limit) = headers
            .get("x-ratelimit-limit")
            .and_then(|x| x.to_str().ok())
        else {
            return;
        };
        let Some(remaining) = headers
            .get("x-ratelimit-remaining")
            .and_then(|x| x.to_str().ok())
        else {
            return;
        };
        let Some(reset_after) = headers
            .get("x-ratelimit-reset-after")
            .and_then(|x| x.to_str().ok())
        else {
            return;
        };
        let Ok(limit) = limit.parse::<u8>() else {
            return;
        };
        let Ok(remaining) = remaining.parse::<u8>() else {
            return;
        };
        let Ok(reset_after) = reset_after.parse::<u64>() else {
            return;
        };

        let mut buckets = self.buckets.lock().unwrap();
        if let Some(bucket) = buckets.get_mut(key) {
            bucket.used = limit.max(remaining) - remaining;
            bucket.reset = Instant::now() + Duration::from_millis(reset_after);
        }
    }
}
