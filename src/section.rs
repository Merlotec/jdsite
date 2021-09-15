use crate::{db, define_uuid_key, user, data};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use std::collections::HashMap;

use maplit::hashmap;

#[derive(Debug, Clone, PartialEq)]
pub struct SectionInfo {
    pub name: String,
    pub subtitle: String,
    pub activities: HashMap<String, Activity>,
    pub image_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityComponent {
    HtmlText(String),
    HtmlFile(String),
    InputItem(InputItem),
}

#[derive(Debug, Clone, PartialEq)]
pub struct InputItem {
    pub title: String,
    pub text: String,
    pub name: String,
    pub ty: FormEntryType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FormEntryType {
    Text {
        placeholder: String,
        rows: u32,
    },
    Checkbox(Vec<String>),
    Radio(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FormEntryData {
    Text(String),
    Index(usize),
    Indices(Vec<usize>),
}

impl FormEntryType {
    pub fn true_false_radio() -> Self {
        Self::Radio(vec!["true".to_owned(), "false".to_owned()])
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Activity {
    pub name: String,
    pub subtitle: String,
    pub activity_url: String,
    pub components: Vec<ActivityComponent>,
}

impl Activity {
    pub fn contains_input_component(&self, name: &str) -> bool {
        for component in self.components.iter() {
            if let ActivityComponent::InputItem(item) = component {
                if item.name == name {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SectionState {
    InProgress,
    Rejected(String),
    InReview(SystemTime),
    Completed,
}

impl ToString for SectionState {
    fn to_string(&self) -> String {
        match self {
            SectionState::InProgress => "In Progress".to_owned(),
            SectionState::Rejected(_) => "Not Approved".to_owned(),
            SectionState::InReview(_) => "In Review".to_owned(),
            SectionState::Completed => "Completed".to_owned(),
        }
    }
}

impl SectionState {
    pub fn is_completed(&self) -> bool {
        if let SectionState::Completed = self {
            true
        } else {
            false
        }
    }

    pub fn css_class(&self) -> String {
        match self {
            SectionState::InProgress => "state-in-progress".to_owned(),
            SectionState::Rejected(_) => "state-rejected".to_owned(),
            SectionState::InReview(_) => "state-in-review".to_owned(),
            SectionState::Completed => "state-completed".to_owned(),
        }
    }

    pub fn css_color(&self) -> String {
        match self {
            SectionState::InProgress => "rgb(175, 175, 175)".to_owned(),
            SectionState::Rejected(_) => "red".to_owned(),
            SectionState::InReview(_) => "orange".to_owned(),
            SectionState::Completed => "green".to_owned(),
        }
    }

    pub fn is_restricted(&self) -> bool {
        match self {
            SectionState::InProgress => false,
            SectionState::Rejected(_) => true,
            SectionState::InReview(_) => false,
            SectionState::Completed => true,
        }
    }

    pub fn time(&self) -> Option<SystemTime> {
        if let SectionState::InReview(time) = self {
            Some(time.clone())
        } else {
            None
        }
    }
}

impl Default for SectionState {
    fn default() -> Self {
        SectionState::InProgress
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub section_index: usize,
    pub award: String,
    pub activity: String,
    pub user_id: user::UserKey,
    pub plan: String,
    pub reflection: String,
    pub input_data: HashMap<String, FormEntryData>,
    pub state: SectionState,
    pub outstanding: bool,
}

impl Section {
    pub fn new(
        section_index: usize,
        award: String,
        activity: String,
        user_id: user::UserKey,
    ) -> Self {
        Self {
            section_index,
            activity,
            award,
            user_id,
            plan: String::new(),
            reflection: String::new(),
            input_data: HashMap::new(),
            state: SectionState::InProgress,
            outstanding: false,
        }
    }

    pub fn get_activity<'a, 'b>(&'a self, data: &'b data::SharedData) -> Option<&'b Activity> {
        if let Some(award) = data.awards.get(&self.award) {
            if let Some(section) = award.sections.get(self.section_index) {
                return section.activities.get(&self.activity);
            }
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AwardInfo {
    pub name: String,
    pub short_name: String,
    pub image_url: String,
    pub sections: [SectionInfo; 6],
}

impl AwardInfo {
    pub fn awards() -> HashMap<String, AwardInfo> {
        hashmap![
            "silver".to_owned() => AwardInfo {
                name: "Silver Senior Duke".to_owned(),
                short_name: "Silver".to_owned(),
                image_url: "/assets/icons/silver.png".to_owned(),
                sections: SectionInfo::silver_sections_list(),
            },
            "gold".to_owned() => AwardInfo {
                name: "Gold Senior Duke".to_owned(),
                short_name: "Gold".to_owned(),
                image_url: "/assets/icons/gold.png".to_owned(),
                sections: SectionInfo::gold_sections_list(),
            },
        ]
    }
}

define_uuid_key!(SectionKey);

pub type SectionDb = db::Database<SectionKey, Section>;

impl SectionInfo {
    pub fn silver_sections_list() -> [SectionInfo; 6] {
        [
            SectionInfo {
                name: "Creative Skills".to_owned(),
                subtitle: "Creative skills to help promote your own well-being".to_owned(),
                image_url: "/section_icons/creative_skills.png".to_owned(),
                activities: hashmap![
                    "video".to_owned() => Activity {
                        name: "Video Editing".to_owned(),
                        subtitle: "Emotional Intelligence & Self Expression".to_owned(),
                        activity_url: "sections/silver/creative/video_editing".to_owned(),
                        components: Vec::new(),
                    },
                    "skill".to_owned() => Activity {
                        name: "Up Your Skill Level".to_owned(),
                        subtitle: "Persistence and Resilience".to_owned(),
                        activity_url: "sections/silver/creative/up_your_skill".to_owned(),
                        components: Vec::new(),
                    },
                    "flashcards".to_owned() => Activity {
                        name: "Flashcards and Mindmaps".to_owned(),
                        subtitle: "Critical Thinking".to_owned(),
                        activity_url: "sections/silver/creative/flashcards".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "Money Skills".to_owned(),
                subtitle: "Financial and negotiation skills for the future".to_owned(),
                image_url: "/section_icons/money_skills.png".to_owned(),
                activities: hashmap![
                    "mend".to_owned() => Activity {
                        name: "Make do and Mend".to_owned(),
                        subtitle: "Persistence and Resilience".to_owned(),
                        activity_url: "sections/silver/money/mend".to_owned(),
                        components: Vec::new(),
                    },
                    "meals".to_owned() => Activity {
                        name: "Five Budget Meals".to_owned(),
                        subtitle: "Budgeting & Cookery".to_owned(),
                        activity_url: "sections/silver/money/meals".to_owned(),
                        components: Vec::new(),
                    },
                    "prices".to_owned() => Activity {
                        name: "Compare Prices".to_owned(),
                        subtitle: "Research & Negotiation Skills".to_owned(),
                        activity_url: "sections/silver/money/compare".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "Home Skills".to_owned(),
                subtitle: "Life skills to help you around the house and beyond".to_owned(),
                image_url: "/section_icons/home_skills.png".to_owned(),
                activities: hashmap![
                    "maintenance".to_owned() => Activity {
                        name: "Maintenance".to_owned(),
                        subtitle: "Problem Solving".to_owned(),
                        activity_url: "sections/silver/home/maintenance".to_owned(),
                        components: Vec::new(),
                    },
                    "tech".to_owned() => Activity {
                        name: "Share your tech Knowledge".to_owned(),
                        subtitle: "Empathy, Adaptability & Mentoring".to_owned(),
                        activity_url: "sections/silver/home/tech".to_owned(),
                        components: Vec::new(),
                    },
                    "bathroom".to_owned() => Activity {
                        name: "Clean the Bathroom".to_owned(),
                        subtitle: "Perseverance".to_owned(),
                        activity_url: "sections/silver/home/bathroom".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "First Aid".to_owned(),
                subtitle: "This is a compulsory challenge with no choices".to_owned(),
                image_url: "/section_icons/first_aid.png".to_owned(),
                activities: hashmap!["first_aid".to_owned() => Activity {
                    name: "First Aid".to_owned(),
                    subtitle: "Critical thinking".to_owned(),
                    activity_url: "sections/silver/first_aid/first_aid".to_owned(),
                    components: Vec::new(),
                }],
            },
            SectionInfo {
                name: "Physical Challenge".to_owned(),
                subtitle: "A challenge to improve fitness and health".to_owned(),
                image_url: "/section_icons/physical_challenge.png".to_owned(),
                activities: hashmap![
                    "run".to_owned() => Activity {
                        name: "Mile Run".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/silver/physical/mile_run".to_owned(),
                        components: Vec::new(),
                    },
                    "walk".to_owned() => Activity {
                        name: "Walk".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/silver/physical/walk".to_owned(),
                        components: Vec::new(),
                    },
                    "bike".to_owned() => Activity {
                        name: "Bike".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/silver/physical/bike".to_owned(),
                        components: Vec::new(),
                    },
                    "swim".to_owned() => Activity {
                        name: "Swim".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/silver/physical/swim".to_owned(),
                        components: Vec::new(),
                    },
                    "machine".to_owned() => Activity {
                        name: "Rowing or other Fitness Machine".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/silver/physical/machine".to_owned(),
                        components: Vec::new(),
                    },
                    "stretch".to_owned() => Activity {
                        name: "Stretch and Relax".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/silver/physical/relax".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "Adventure Challenge".to_owned(),
                subtitle: "An adventure to enjoy and challenge you".to_owned(),
                image_url: "/section_icons/adventure_challenge.png".to_owned(),
                activities: hashmap![
                    "trip".to_owned() => Activity {
                        name: "Outdoor Day Trips".to_owned(),
                        subtitle: "Decision Making & Navigation".to_owned(),
                        activity_url: "sections/silver/adventure/day_trip".to_owned(),
                        components: Vec::new(),
                    },
                    "camping".to_owned() => Activity {
                        name: "Go Camping".to_owned(),
                        subtitle: "Unplugging, Survival & Groundedness".to_owned(),
                        activity_url: "sections/silver/adventure/camping".to_owned(),
                        components: Vec::new(),
                    },
                    "climb".to_owned() => Activity {
                        name: "Climb Ben Nevis Challenge".to_owned(),
                        subtitle: "Problem Solving & Perseverance".to_owned(),
                        activity_url: "sections/silver/adventure/ben_nevis".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
        ]
    }

    pub fn gold_sections_list() -> [SectionInfo; 6] {
        [
            SectionInfo {
                name: "Creative Skills".to_owned(),
                subtitle: "Creative skills to help promote your own well-being".to_owned(),
                image_url: "/section_icons/creative_skills.png".to_owned(),
                activities: hashmap![
                    "skill".to_owned() => Activity {
                        name: "Up Your Skill Level".to_owned(),
                        subtitle: "Persistence and Resilience".to_owned(),
                        activity_url: "sections/gold/creative/up_your_skill".to_owned(),
                        components: Vec::new(),
                    },
                    "trailer".to_owned() => Activity {
                        name: "Trailer Making".to_owned(),
                        subtitle: "Organisation & Prioritisation".to_owned(),
                        activity_url: "sections/gold/creative/trailer_making".to_owned(),
                        components: Vec::new(),
                    },
                    "brand".to_owned() => Activity {
                        name: "Create your Brand".to_owned(),
                        subtitle: "Self awareness & Effective Communication".to_owned(),
                        activity_url: "sections/gold/creative/brand".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "Money Skills".to_owned(),
                subtitle: "Financial and negotiation skills for the future".to_owned(),
                image_url: "/section_icons/money_skills.png".to_owned(),
                activities: hashmap![
                    "coupons".to_owned() => Activity {
                        name: "Coupons".to_owned(),
                        subtitle: "Thriftiness".to_owned(),
                        activity_url: "sections/gold/money/coupons".to_owned(),
                        components: Vec::new(),
                    },
                    "live_for_less".to_owned() => Activity {
                        name: "Live for Less".to_owned(),
                        subtitle: "Budgeting and Cooperating".to_owned(),
                        activity_url: "sections/gold/money/live_for_less".to_owned(),
                        components: Vec::new(),
                    },
                    "saving".to_owned() => Activity {
                        name: "Saving".to_owned(),
                        subtitle: "Communication, Cooperation, Time Management".to_owned(),
                        activity_url: "sections/gold/money/saving".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "Home Skills".to_owned(),
                subtitle: "Life skills to help you around the house and beyond".to_owned(),
                image_url: "/section_icons/home_skills.png".to_owned(),
                activities: hashmap![
                    "letter".to_owned() => Activity {
                        name: "Letter Writing".to_owned(),
                        subtitle: "Assertiveness".to_owned(),
                        activity_url: "sections/gold/home/letter".to_owned(),
                        components: Vec::new(),
                    },
                    "food".to_owned() => Activity {
                        name: "Food Choices and Safety".to_owned(),
                        subtitle: "Health and Self Awareness".to_owned(),
                        activity_url: "sections/gold/home/food_safety".to_owned(),
                        components: Vec::new(),
                    },
                    "clean".to_owned() => Activity {
                        name: "Spring Clean".to_owned(),
                        subtitle: "Persistence".to_owned(),
                        activity_url: "sections/gold/home/spring_clean".to_owned(),
                        components: Vec::new(),
                    },
                    "drink".to_owned() => Activity {
                        name: "Drink Choices".to_owned(),
                        subtitle: "Health and Safety, Awareness of Peer Pressure".to_owned(),
                        activity_url: "sections/gold/home/drink".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "First Aid".to_owned(),
                subtitle: "This is a compulsory challenge with no choices".to_owned(),
                image_url: "/section_icons/first_aid.png".to_owned(),
                activities: hashmap!["first_aid".to_owned() => Activity {
                    name: "First Aid".to_owned(),
                    subtitle: "Critical thinking".to_owned(),
                    activity_url: "sections/silver/first_aid/first_aid".to_owned(),
                    components: Vec::new(),
                }],
            },
            SectionInfo {
                name: "Physical Challenge".to_owned(),
                subtitle: "A challenge to improve fitness and health".to_owned(),
                image_url: "/section_icons/physical_challenge.png".to_owned(),
                activities: hashmap![
                    "run".to_owned() => Activity {
                        name: "5km Run".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/gold/physical/run".to_owned(),
                        components: Vec::new(),
                    },
                    "walk".to_owned() => Activity {
                        name: "5km Walk".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/gold/physical/walk".to_owned(),
                        components: Vec::new(),
                    },
                    "bike".to_owned() => Activity {
                        name: "10km Bike".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/gold/physical/bike".to_owned(),
                        components: Vec::new(),
                    },
                    "swim".to_owned() => Activity {
                        name: "Swim".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/gold/physical/swim".to_owned(),
                        components: Vec::new(),
                    },
                    "machine".to_owned() => Activity {
                        name: "Rowing or other Fitness Machine".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/gold/physical/machine".to_owned(),
                        components: Vec::new(),
                    },
                    "stretch".to_owned() => Activity {
                        name: "Stretch and Relax".to_owned(),
                        subtitle: "Perseverance, Time Management & Health Behaviour".to_owned(),
                        activity_url: "sections/gold/physical/relax".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
            SectionInfo {
                name: "Adventure Challenge".to_owned(),
                subtitle: "An adventure to enjoy and challenge you".to_owned(),
                image_url: "/section_icons/adventure_challenge.png".to_owned(),
                activities: hashmap![
                    "camping".to_owned() => Activity {
                        name: "Go Camping".to_owned(),
                        subtitle: "Unplugging, Survival & Groundedness".to_owned(),
                        activity_url: "sections/gold/adventure/camping".to_owned(),
                        components: Vec::new(),
                    },
                    "tour".to_owned() => Activity {
                        name: "Walking Tour / Quiz".to_owned(),
                        subtitle: "Problem Solving & Creativity".to_owned(),
                        activity_url: "sections/gold/adventure/walking_tour".to_owned(),
                        components: Vec::new(),
                    },
                    "adventure".to_owned() => Activity {
                        name: "Map Reading, Adventure Day".to_owned(),
                        subtitle: "Planning, Adventurous spirit, Navigation".to_owned(),
                        activity_url: "sections/gold/adventure/adventure_day".to_owned(),
                        components: Vec::new(),
                    },
                ],
            },
        ]
    }
}
