use crate::config::Settings;
use crate::storage::{Activity, Database, Session};
use chrono::{DateTime, TimeZone, Utc};
use parking_lot::RwLock;
use std::sync::Arc;
use tracing::info;

pub struct TimelineManager {
    db: Arc<Database>,
    settings: Arc<RwLock<Settings>>,
}

impl TimelineManager {
    pub fn new(db: Arc<Database>, settings: Arc<RwLock<Settings>>) -> Self {
        Self { db, settings }
    }

    pub fn get_timeline(
        &self,
        days: usize,
    ) -> Result<Timeline, Box<dyn std::error::Error + Send + Sync>> {
        let now = Utc::now().timestamp();
        let start = now - (days as i64 * 24 * 60 * 60);

        let activities = self.db.get_activities_in_range(start, now)?;
        let sessions = self.db.get_sessions(100)?;

        let mut timeline_days: Vec<TimelineDay> = Vec::new();

        let mut current_day: Option<String> = None;
        let mut day_activities: Vec<TimelineActivity> = Vec::new();

        for activity in activities {
            let dt = Utc
                .timestamp_opt(activity.timestamp, 0)
                .single()
                .unwrap_or_else(Utc::now);
            let day_key = dt.format("%Y-%m-%d").to_string();

            if current_day.as_ref() != Some(&day_key) {
                if !day_activities.is_empty() {
                    let day_activities_clone = day_activities.clone();
                    timeline_days.push(TimelineDay {
                        date: current_day.unwrap_or_default(),
                        activities: day_activities_clone,
                    });
                    day_activities.clear();
                }
                current_day = Some(day_key);
            }

            day_activities.push(TimelineActivity {
                id: activity.id.clone(),
                activity_type: activity.activity_type.clone(),
                title: activity
                    .title
                    .clone()
                    .or_else(|| activity.content.clone())
                    .or_else(|| activity.app_name.clone())
                    .unwrap_or_default(),
                app_name: activity.app_name.clone(),
                timestamp: activity.timestamp,
                session_id: activity.session_id.clone(),
            });
        }

        if !day_activities.is_empty() {
            timeline_days.push(TimelineDay {
                date: current_day.unwrap_or_default(),
                activities: day_activities,
            });
        }

        let timeline_sessions: Vec<TimelineSession> = sessions
            .iter()
            .map(|s| TimelineSession {
                id: s.id.clone(),
                session_type: s.session_type.clone(),
                start_time: s.start_time,
                end_time: s.end_time,
                app_sequence: s.app_sequence.clone(),
            })
            .collect();

        Ok(Timeline {
            days: timeline_days,
            sessions: timeline_sessions,
        })
    }

    pub fn get_session_activities(
        &self,
        session_id: &str,
    ) -> Result<Vec<Activity>, Box<dyn std::error::Error + Send + Sync>> {
        let all_activities = self.db.get_activities(10000, 0)?;

        let session_activities: Vec<Activity> = all_activities
            .into_iter()
            .filter(|a| a.session_id.as_deref() == Some(session_id))
            .collect();

        Ok(session_activities)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Timeline {
    pub days: Vec<TimelineDay>,
    pub sessions: Vec<TimelineSession>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineDay {
    pub date: String,
    pub activities: Vec<TimelineActivity>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineActivity {
    pub id: String,
    pub activity_type: String,
    pub title: String,
    pub app_name: Option<String>,
    pub timestamp: i64,
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TimelineSession {
    pub id: String,
    pub session_type: String,
    pub start_time: i64,
    pub end_time: Option<i64>,
    pub app_sequence: Option<String>,
}
