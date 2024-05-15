use std::collections::HashSet;

use chrono::{DateTime, Local};
use log::warn;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use strum::EnumIter;

use super::{
    character::Class, items::*, tavern::Location, unlockables::HabitatType,
    CCGet, CFPGet, CGet,
};
use crate::{command::AttributeType, error::SFError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[allow(missing_docs)]
/// The type of a reward you can win by spinning the wheel. The wheel can be
/// upgraded, so some rewards may not always eb available
pub enum WheelRewardType {
    Mushrooms,
    Stone,
    StoneXL,
    Wood,
    WoodXL,
    Experience,
    ExperienceXL,
    Silver,
    SilverXL,
    Arcane,
    Souls,
    Item,
    PetItem(PetItem),
    Unknown,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The thing you won from spinning the wheel
pub struct WheelReward {
    /// The type of item you have won
    pub typ: WheelRewardType,
    /// The amount of the type you have won
    pub amount: i64,
}

impl WheelReward {
    pub(crate) fn parse(
        data: &[i64],
        upgraded: bool,
    ) -> Result<WheelReward, SFError> {
        let raw_typ = data.cget(0, "wheel reward typ")?;
        let mut amount = data.cget(1, "wheel reward amount")?;
        // NOTE: I have only tested upgraded and infered not upgraded from that
        let typ = match raw_typ {
            0 => WheelRewardType::Mushrooms,
            1 => {
                if upgraded {
                    WheelRewardType::Arcane
                } else {
                    WheelRewardType::Wood
                }
            }
            2 => WheelRewardType::ExperienceXL,
            3 => {
                if upgraded {
                    let res = WheelRewardType::PetItem(
                        PetItem::parse(amount).ok_or_else(|| {
                            SFError::ParsingError(
                                "pet wheel reward type",
                                amount.to_string(),
                            )
                        })?,
                    );
                    amount = 1;
                    res
                } else {
                    WheelRewardType::Stone
                }
            }
            4 => WheelRewardType::SilverXL,
            5 => {
                // The amount does not seem to do anything.
                // 1 => equipment
                // 2 => potion
                amount = 1;
                WheelRewardType::Item
            }
            6 => WheelRewardType::WoodXL,
            7 => WheelRewardType::Experience,
            8 => WheelRewardType::StoneXL,
            9 => {
                if upgraded {
                    WheelRewardType::Souls
                } else {
                    WheelRewardType::Silver
                }
            }
            x => {
                warn!("unknown wheel reward type: {x}");
                WheelRewardType::Unknown
            }
        };
        Ok(WheelReward { typ, amount })
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// A possible reward on the calendar
pub struct CalendarReward {
    /// Note that this is technically correct, but at low levels, these are
    /// often overwritten to silver
    // FIXME: figure out how exactly
    pub typ: CalendarRewardType,
    /// The mount of the type this reward yielded
    pub amount: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
/// The type of reward gainable by collecting the calendar
pub enum CalendarRewardType {
    Silver,
    Mushrooms,
    Experience,
    Wood,
    Stone,
    Souls,
    Arcane,
    Runes,
    Item,
    Attribute(AttributeType),
    Fruit(HabitatType),
    Level,
    Potion(PotionType),
    TenQuicksandGlasses,
    LevelUp,
    Unknown,
}

impl CalendarReward {
    pub(crate) fn parse(data: &[i64]) -> Result<CalendarReward, SFError> {
        let amount = data.cget(1, "c reward amount")?;
        let typ = data.cget(0, "c reward typ")?;
        let typ = match typ {
            1 => CalendarRewardType::Silver,
            2 => CalendarRewardType::Mushrooms,
            3 => CalendarRewardType::Experience,
            4 => CalendarRewardType::Wood,
            5 => CalendarRewardType::Stone,
            6 => CalendarRewardType::Souls,
            7 => CalendarRewardType::Arcane,
            8 => CalendarRewardType::Runes,
            10 => CalendarRewardType::Item,
            11 => CalendarRewardType::Attribute(AttributeType::Strength),
            12 => CalendarRewardType::Attribute(AttributeType::Dexterity),
            13 => CalendarRewardType::Attribute(AttributeType::Intelligence),
            14 => CalendarRewardType::Attribute(AttributeType::Constitution),
            15 => CalendarRewardType::Attribute(AttributeType::Luck),
            x @ 16..=20 => {
                if let Some(typ) = HabitatType::from_typ_id(x - 15) {
                    CalendarRewardType::Fruit(typ)
                } else {
                    warn!("unknown pet class in c rewards");
                    CalendarRewardType::Unknown
                }
            }
            21 => CalendarRewardType::LevelUp,
            22 => CalendarRewardType::Potion(PotionType::EternalLife),
            23 => CalendarRewardType::TenQuicksandGlasses,
            24 => CalendarRewardType::Potion(PotionType::Strength),
            25 => CalendarRewardType::Potion(PotionType::Dexterity),
            26 => CalendarRewardType::Potion(PotionType::Intelligence),
            27 => CalendarRewardType::Potion(PotionType::Constitution),
            28 => CalendarRewardType::Potion(PotionType::Luck),
            x => {
                warn!("Unknown calendar reward: {x}");
                CalendarRewardType::Unknown
            }
        };

        Ok(CalendarReward { typ, amount })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Everything, that changes over time
pub struct TimedSpecials {
    /// All of the events active in the tavern
    pub events: Events,
    /// The stuff you can do for bonus rewards
    pub tasks: Tasks,
    /// Grants rewards once a day
    pub calendar: Calendar,
    /// Dr. Abawuwu's wheel
    pub wheel: Wheel,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Information about the events active in the tavern
pub struct Events {
    /// All of the events active in the tavern
    pub active: HashSet<Event>,
    /// The time at which all of the events end. Mostly just Sunday 23:59.
    pub ends: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[doc(alias = "DailyLoginBonus")]
/// Grants rewards once a day
pub struct Calendar {
    /// The amount of times the calendar has been collected already.
    /// `rewards[collected]` will give you the position in the rewards you will
    /// get for collecting today (if you can)
    pub collected: usize,
    /// The things you can get from the calendar
    pub rewards: Vec<CalendarReward>,
    /// The time at which the calendar door wll be unlocked. If this is in the
    /// past, that means it is available to open
    pub next_possible: Option<DateTime<Local>>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// The tasks you get from the goblin gleeman
pub struct Tasks {
    /// The tasks, that update daily
    pub daily: DailyTasks,
    /// The tasks, that follow some server wide theme
    pub event: EventTasks,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Information about the tasks, that reset every day
pub struct DailyTasks {
    /// The tasks you have to do
    pub tasks: Vec<DailyTask>,
    /// The rewards available for completing tasks.
    pub rewards: [RewardChest; 3],
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Information about the tasks, that are based on some event theme
pub struct EventTasks {
    /// The "theme" the event task has. This is mainly irrelevant
    pub theme: EventTasksTheme,
    /// The time at which the event tasks have been set
    pub start: Option<DateTime<Local>>,
    /// The time at which the event tasks will reset
    pub end: Option<DateTime<Local>>,
    /// The actual tasks you have to complete
    pub tasks: Vec<EventTask>,
    /// The rewards available for completing tasks.
    pub rewards: [RewardChest; 3],
}

macro_rules! impl_tasks {
    ($t:ty) => {
        impl $t {
            /// The amount of tasks you have collected
            #[must_use]
            pub fn completed(&self) -> usize {
                self.tasks.iter().filter(|a| a.is_completed()).count()
            }

            /// The amount of points you have collected from completing tasks
            #[must_use]
            pub fn earned_points(&self) -> u32 {
                self.tasks
                    .iter()
                    .filter(|a| a.is_completed())
                    .map(|a| a.point_reward)
                    .sum()
            }
            /// The amount of points, that are available in total
            #[must_use]
            pub fn total_points(&self) -> u32 {
                self.tasks.iter().map(|a| a.point_reward).sum()
            }
        }
    };
}

impl_tasks!(DailyTasks);
impl_tasks!(EventTasks);

macro_rules! impl_task {
    ($t:ty) => {
        impl $t {
            /// The amount of tasks you have collected
            #[must_use]
            pub fn is_completed(&self) -> bool {
                self.current >= self.target
            }
        }
    };
}

impl_task!(EventTask);
impl_task!(DailyTask);

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Dr. Abawuwu's wheel
pub struct Wheel {
    /// The amount of lucky coins you have to spin the weel
    pub lucky_coins: u32,
    /// The amount of times you have spun the wheel today already (0 -> 20)
    pub spins_today: u8,
    /// The next time you can spin the wheel for free
    pub next_free_spin: Option<DateTime<Local>>,
    /// The result of spinning the wheel
    pub result: Option<WheelReward>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, FromPrimitive, Default, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
/// The theme the event tasks have
pub enum EventTasksTheme {
    ShoppingSpree = 4,
    TimeSkipper = 5,
    RuffianReset = 6,
    PartTimeNudist = 7,
    Scrimper = 8,
    Scholar = 9,
    UnderworldFigure = 11,
    #[default]
    Unknown = 245,
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
/// The type of task you have to do
pub enum EventTaskTyp {
    LureHeroesIntoUnderworld,
    WinFightsAgainst(Class),
    WinFightsBareHands,
    SpendGoldInShop,
    SpendGoldOnUpgrades,
    RequestNewGoods,
    BuyHourGlasses,
    SkipQuest,
    SkipGameOfDiceWait,
    WinFights,
    WinFightsBackToBack,
    WinFightsNoChestplate,
    WinFightsNoGear,
    WinFightsNoEpicsLegendaries,
    EarnMoneyCityGuard,
    EarnMoneyFromHoFFights,
    EarnMoneySellingItems,
    ColectGoldFromPit,
    GainXpFromQuests,
    GainXpFromAcademy,
    GainXpFromArenaFights,
    GainXpFromAdventuromatic,
    ClaimSoulsFromExtractor,
    FillMushroomsInAdventuromatic,
    Unknown,
}

impl EventTaskTyp {
    pub(crate) fn parse(num: i64) -> EventTaskTyp {
        match num {
            12 => EventTaskTyp::LureHeroesIntoUnderworld,
            48 => EventTaskTyp::WinFightsAgainst(Class::Warrior),
            49 => EventTaskTyp::WinFightsAgainst(Class::Mage),
            50 => EventTaskTyp::WinFightsAgainst(Class::Scout),
            51 => EventTaskTyp::WinFightsAgainst(Class::Assassin),
            52 => EventTaskTyp::WinFightsAgainst(Class::Druid),
            53 => EventTaskTyp::WinFightsAgainst(Class::Bard),
            54 => EventTaskTyp::WinFightsAgainst(Class::BattleMage),
            55 => EventTaskTyp::WinFightsAgainst(Class::Berserker),
            56 => EventTaskTyp::WinFightsAgainst(Class::DemonHunter),
            57 => EventTaskTyp::WinFightsBareHands,
            65 => EventTaskTyp::SpendGoldInShop,
            66 => EventTaskTyp::SpendGoldOnUpgrades,
            67 => EventTaskTyp::RequestNewGoods,
            68 => EventTaskTyp::BuyHourGlasses,
            69 => EventTaskTyp::SkipQuest,
            70 => EventTaskTyp::SkipGameOfDiceWait,
            71 => EventTaskTyp::WinFights,
            72 => EventTaskTyp::WinFightsBackToBack,
            75 => EventTaskTyp::WinFightsNoChestplate,
            76 => EventTaskTyp::WinFightsNoGear,
            77 => EventTaskTyp::WinFightsNoEpicsLegendaries,
            78 => EventTaskTyp::EarnMoneyCityGuard,
            79 => EventTaskTyp::EarnMoneyFromHoFFights,
            80 => EventTaskTyp::EarnMoneySellingItems,
            81 => EventTaskTyp::ColectGoldFromPit,
            82 => EventTaskTyp::GainXpFromQuests,
            83 => EventTaskTyp::GainXpFromAcademy,
            84 => EventTaskTyp::GainXpFromArenaFights,
            85 => EventTaskTyp::GainXpFromAdventuromatic,
            90 => EventTaskTyp::ClaimSoulsFromExtractor,
            91 => EventTaskTyp::FillMushroomsInAdventuromatic,
            92 => EventTaskTyp::WinFightsAgainst(Class::Necromancer),
            x => {
                warn!("Unknown event task typ: {x}");
                EventTaskTyp::Unknown
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EventTask {
    pub typ: EventTaskTyp,
    pub current: u64,
    pub target: u64,
    pub point_reward: u32,
}

impl EventTask {
    pub(crate) fn parse(data: &[i64]) -> Result<EventTask, SFError> {
        let raw_typ = data.cget(0, "event task typ")?;
        Ok(EventTask {
            typ: EventTaskTyp::parse(raw_typ),
            current: data.csiget(1, "current eti", 0)?,
            target: data.csiget(2, "target eti", u64::MAX)?,
            point_reward: data.csiget(3, "reward eti", 0)?,
        })
    }
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RewardChest {
    pub opened: bool,
    pub reward: [Option<Reward>; 2],
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Reward {
    pub typ: RewardTyp,
    pub amount: u64,
}

impl Reward {
    pub(crate) fn parse(data: &[i64]) -> Result<Reward, SFError> {
        Ok(Reward {
            typ: data
                .cfpget(0, "reward typ", |a| a)?
                .unwrap_or(RewardTyp::Unknown),
            amount: data.csiget(1, "reward amount", 0)?,
        })
    }
}

impl RewardChest {
    pub(crate) fn parse(data: &[i64]) -> Result<RewardChest, SFError> {
        let mut reward: [Option<Reward>; 2] = Default::default();

        let indices: &[usize] = match data.len() {
            5 => &[3],
            _ => &[3, 5],
        };

        for (i, reward) in indices.iter().copied().zip(&mut reward) {
            *reward = Some(Reward::parse(&data[i..])?);
        }

        Ok(RewardChest {
            opened: data[0] == 1,
            reward,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, FromPrimitive, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RewardTyp {
    ExtraBeer = 2,
    Mushroom = 3,
    Silver = 4,
    LuckyCoins = 5,
    Stone = 9,
    Souls = 10,
    Experience = 24,
    Hourglass = 26,
    Beer = 28,
    Unknown = 999,
}

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq, Hash, EnumIter)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
/// The type of event, that is currently happening on the server
pub enum Event {
    ExceptionalXPEvent = 0,
    GloriousGoldGalore,
    TidyToiletTime,
    AssemblyOfAwesomeAnimals,
    FantasticFortressFestivity,
    DaysOfDoomedSouls,
    WitchesDance,
    SandsOfTimeSpecial,
    ForgeFrenzyFestival,
    EpicShoppingSpreeExtravaganza,
    EpicQuestExtravaganza,
    EpicGoodLuckExtravaganza,
    OneBeerTwoBeerFreeBeer,
    PieceworkParty,
    LuckyDay,
    CrazyMushroomHarvest,
    HollidaySale,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
/// Something you have to do to get bells
pub struct DailyTask {
    /// The thing you have to do to get points
    pub typ: DailyTaskType,
    /// The amount of `typ` you have currently alredy done
    pub current: u64,
    /// The amount of `typ` you have to do to get the points
    pub target: u64,
    /// The amount of points/bells you get
    pub point_reward: u32,
}

impl DailyTask {
    pub(crate) fn parse(data: &[i64]) -> Result<Self, SFError> {
        Ok(DailyTask {
            typ: DailyTaskType::parse(data.cget(0, "daily task type")?),
            current: data.csiget(1, "daily current", 0)?,
            target: data.csiget(2, "daily target", 999)?,
            point_reward: data.csiget(3, "daily bells", 0)?,
        })
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(missing_docs)]
/// The type of quest you have to complete to get the points of a daily task
pub enum DailyTaskType {
    DrinkBeer,
    FindGemInFortress,
    ConsumeThirstForAdventure,
    FightGuildHydra,
    FightGuildPortal,
    SpinWheelOfFortune,
    FeedPets,
    FightOtherPets,
    BlacksmithDismantle,
    ThrowItemInToilet,
    PlayDice,
    LureHeoesInUnderworld,
    EnterDemonPortal,
    GuildReadyFight,
    SacrificeRunes,
    TravelTo(Location),
    WinFights(Option<Class>),
    DefeatOtherPet,
    ThrowItemInCauldron,
    WinFightsWithBareHands,
    DefeatGambler,
    Upgrade(AttributeType),
    ConsumeThirstFromUnderworld,
    UpgradeArenaManager,
    ThrowEpicInToilet,
    BuyOfferFromArenaManager,
    FightInPetHabitat,
    WinFightsWithoutEpics,
    Unknown,
}

impl DailyTaskType {
    pub(crate) fn parse(val: i64) -> DailyTaskType {
        match val {
            1 => DailyTaskType::DrinkBeer,
            2 => DailyTaskType::ConsumeThirstForAdventure,
            3 => DailyTaskType::WinFights(None),
            4 => DailyTaskType::SpinWheelOfFortune,
            5 => DailyTaskType::FightGuildHydra,
            6 => DailyTaskType::FightGuildPortal,
            7 => DailyTaskType::FeedPets,
            8 => DailyTaskType::FightOtherPets,
            9 => DailyTaskType::BlacksmithDismantle,
            10 => DailyTaskType::ThrowItemInToilet,
            11 => DailyTaskType::PlayDice,
            12 => DailyTaskType::LureHeoesInUnderworld,
            13 => DailyTaskType::EnterDemonPortal,
            14 => DailyTaskType::DefeatGambler,
            15 => DailyTaskType::Upgrade(AttributeType::Strength),
            16 => DailyTaskType::Upgrade(AttributeType::Dexterity),
            17 => DailyTaskType::Upgrade(AttributeType::Intelligence),
            18 => DailyTaskType::ConsumeThirstFromUnderworld,
            19 => DailyTaskType::GuildReadyFight,
            20 => DailyTaskType::FindGemInFortress,
            21 => DailyTaskType::ThrowItemInCauldron,
            22 => DailyTaskType::FightInPetHabitat,
            23 => DailyTaskType::UpgradeArenaManager,
            24 => DailyTaskType::SacrificeRunes,
            25..=45 => {
                let Some(location) = FromPrimitive::from_i64(val - 24) else {
                    return DailyTaskType::Unknown;
                };
                DailyTaskType::TravelTo(location)
            }
            46 => DailyTaskType::ThrowEpicInToilet,
            47 => DailyTaskType::BuyOfferFromArenaManager,
            48 => DailyTaskType::WinFights(Some(Class::Warrior)),
            49 => DailyTaskType::WinFights(Some(Class::Mage)),
            50 => DailyTaskType::WinFights(Some(Class::Scout)),
            51 => DailyTaskType::WinFights(Some(Class::Assassin)),
            52 => DailyTaskType::WinFights(Some(Class::Druid)),
            53 => DailyTaskType::WinFights(Some(Class::Bard)),
            54 => DailyTaskType::WinFights(Some(Class::BattleMage)),
            55 => DailyTaskType::WinFights(Some(Class::Berserker)),
            56 => DailyTaskType::WinFights(Some(Class::DemonHunter)),
            57 => DailyTaskType::WinFightsWithBareHands,
            58 => DailyTaskType::DefeatOtherPet,
            77 => DailyTaskType::WinFightsWithoutEpics,
            92 => DailyTaskType::WinFights(Some(Class::Necromancer)),
            x => {
                warn!("Unknown daily quest: {x}");
                DailyTaskType::Unknown
            }
        }
    }
}
