
use crate::{db, define_uuid_key};
use serde::{Serialize, Deserialize};


pub struct SectionInfo {
    pub name: String,
    pub subtitle: String,
    pub activities: Vec<Activity>,
    pub image_url: String,
}

pub struct Activity {
    pub name: String,
    pub subtitle: String,
    pub activity_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SectionState {
    InProgress,
    InReview,
    Completed,
}

impl Default for SectionState {
    fn default() -> Self {
        SectionState::InProgress
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Section {
    pub description: String,
    pub pre_text: String,
    pub post_text: String,
    pub assets: Vec<String>,
    pub state: SectionState,
}

define_uuid_key!(SectionKey);

pub type SectionDb = db::Database<SectionKey, Section>;

impl SectionInfo {
    pub fn sections_list() -> [SectionInfo; 6] {
        [
            SectionInfo {
                name: "Creative Skills".to_owned(),
                subtitle: "Creative skills to help promote your own well-being".to_owned(),
                image_url: "/section_icons/creative_skills.png".to_owned(),
                activities: vec![
                    Activity {
                        name: "Video Editing".to_owned(),
                        subtitle: "Emotional Intelligence & Self Expression".to_owned(),
                        activity_url: "sections/creative/video_editing".to_owned(),
                    },
                    Activity {
                        name: "Up Your Skill Level".to_owned(),
                        subtitle: "Persistence and Resilience".to_owned(),
                        activity_url: "sections/creative/up_your_skill".to_owned(),
                    },
                    Activity {
                        name: "Trailer Making".to_owned(),
                        subtitle: "Organisation & Prioritisation".to_owned(),
                        activity_url: "sections/creative/trailer_making".to_owned(),
                    },
                    Activity {
                        name: "Create your Brand".to_owned(),
                        subtitle: "Self awareness & Effective Communication".to_owned(),
                        activity_url: "sections/creative/brand".to_owned(),
                    },
                    Activity {
                        name: "Flashcards and Mindmaps".to_owned(),
                        subtitle: "Critical Thinking".to_owned(),
                        activity_url: "sections/creative/flashcards".to_owned(),
                    },
                ],
            },
            SectionInfo {
                name: "Money Skills".to_owned(),
                subtitle: "Financial and negotiation skills for the future".to_owned(),
                image_url: "/section_icons/money_skills.png".to_owned(),
                activities: vec![
                    Activity {
                        name: "Saving".to_owned(),
                        subtitle: "Communication, Cooperation, coping with monotony, time management".to_owned(),
                        activity_url: "sections/money/saving".to_owned(),
                    },
                    Activity {
                        name: "Make do and Mend".to_owned(),
                        subtitle: "Persistence and Resilience".to_owned(),
                        activity_url: "sections/money/mend".to_owned(),
                    },
                    Activity {
                        name: "Coupons".to_owned(),
                        subtitle: "Thriftiness".to_owned(),
                        activity_url: "sections/money/coupons".to_owned(),
                    },
                    Activity {
                        name: "Compare Prices".to_owned(),
                        subtitle: "Research & Negotiation Skills".to_owned(),
                        activity_url: "sections/money/compare".to_owned(),
                    },
                    Activity {
                        name: "Five Budget Meals".to_owned(),
                        subtitle: "Budgeting & Cookery".to_owned(),
                        activity_url: "sections/money/meals".to_owned(),
                    },
                ],
            },
            SectionInfo {
                name: "Home Skills".to_owned(),
                subtitle: "Life skills to help you around the house and beyond".to_owned(),
                image_url: "/section_icons/home_skills.png".to_owned(),
                activities: vec![
                    Activity {
                        name: "Maintenance".to_owned(),
                        subtitle: "Problem Solving".to_owned(),
                        activity_url: "sections/home/maintenance".to_owned(),
                    },
                    Activity {
                        name: "Share your tech Knowledge".to_owned(),
                        subtitle: "Empathy, Adaptability & Mentoring".to_owned(),
                        activity_url: "sections/home/tech".to_owned(),
                    },
                    Activity {
                        name: "Letter Writing".to_owned(),
                        subtitle: "Assertiveness".to_owned(),
                        activity_url: "sections/home/letter".to_owned(),
                    },
                    Activity {
                        name: "Food Choices and Safety".to_owned(),
                        subtitle: "Health and Self Awareness".to_owned(),
                        activity_url: "sections/home/food_safety".to_owned(),
                    },
                    Activity {
                        name: "Clean the Bathroom".to_owned(),
                        subtitle: "Perseverance".to_owned(),
                        activity_url: "sections/home/bathroom".to_owned(),
                    },
                    Activity {
                        name: "Spring Clean".to_owned(),
                        subtitle: "Persistence".to_owned(),
                        activity_url: "sections/home/spring_clean".to_owned(),
                    },
                ],
            },
            SectionInfo {
                name: "First Aid".to_owned(),
                subtitle: "This is a compulsory challenge with no choices".to_owned(),
                image_url: "/section_icons/first_aid.png".to_owned(),
                activities: vec![
                    Activity {
                        name: "First Aid".to_owned(),
                        subtitle: "Critical thinking".to_owned(),
                        activity_url: "sections/first_aid/first_aid".to_owned(),
                    },
                ],
            },
            SectionInfo {
                name: "Physical Challenge".to_owned(),
                subtitle: "A challenge to improve fitness and health".to_owned(),
                image_url: "/section_icons/physical_challenge.png".to_owned(),
                activities: vec![
                    Activity {
                        name: "Mile Run".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/physical/mile_run".to_owned(),
                    },
                    Activity {
                        name: "Walk".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/physical/walk".to_owned(),
                    },
                    Activity {
                        name: "Bike".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/physical/bike".to_owned(),
                    },
                    Activity {
                        name: "Swim".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/physical/swim".to_owned(),
                    },
                    Activity {
                        name: "Rowing or other Fitness Machine".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/physical/machine".to_owned(),
                    },
                    Activity {
                        name: "Stretch and Relax".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/physical/relax".to_owned(),
                    },
                ],
            },
            SectionInfo {
                name: "Adventure Challenge".to_owned(),
                subtitle: "An adventure to enjoy and challenge you".to_owned(),
                image_url: "/section_icons/adventure_challenge.png".to_owned(),
                activities: vec![
                    Activity {
                        name: "Outdoor Day Trips".to_owned(),
                        subtitle: "Decision Making & Navigation".to_owned(),
                        activity_url: "sections/adventure/day_trip".to_owned(),
                    },
                    Activity {
                        name: "Go Camping".to_owned(),
                        subtitle: "Unplugging, Survival & Groundedness".to_owned(),
                        activity_url: "sections/adventure/camping".to_owned(),
                    },
                    Activity {
                        name: "Climb Ben Nevis Challenge".to_owned(),
                        subtitle: "Problem Solving & Perseverance".to_owned(),
                        activity_url: "sections/adventure/ben_nevis".to_owned(),
                    },
                    Activity {
                        name: "Walking Tour".to_owned(),
                        subtitle: "Problem Solving & Creativity".to_owned(),
                        activity_url: "sections/adventure/walking_tour".to_owned(),
                    },
                ],
            },
        ]
    }
}