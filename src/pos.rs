use harper_core::DictWordMetadata;

pub type PosPredicate = fn(&DictWordMetadata) -> bool;

#[derive(Eq, Hash, PartialEq)]
pub enum POS {
    Adjective,
    Adverb,
    Conjunction,
    Determiner,
    Noun,
    Preposition,
    Pronoun,
    ProperNoun,
    Verb,
}

pub struct POSInfo {
    pub _name: &'static str,
    pub _ptb: &'static str, // Penn Treebank
    pub letter: &'static str,
    pub _emoji: &'static str,
}

pub const POS_DEFINITIONS: &[(POS, PosPredicate)] = &[
    (POS::Noun, |m| m.is_noun() && !m.is_proper_noun()),
    (POS::ProperNoun, DictWordMetadata::is_proper_noun),
    (POS::Verb, DictWordMetadata::is_verb),
    (POS::Adjective, DictWordMetadata::is_adjective),
    (POS::Adverb, DictWordMetadata::is_adverb),
    (POS::Conjunction, DictWordMetadata::is_conjunction),
    (POS::Determiner, DictWordMetadata::is_determiner),
    (POS::Preposition, |m| m.preposition),
    (POS::Pronoun, DictWordMetadata::is_pronoun),
];

pub fn pos_info(pos: &POS) -> POSInfo {
    match pos {
        POS::Noun => POSInfo {
            letter: "N",
            _ptb: "NN",
            _emoji: "📦",
            _name: "noun",
        },
        POS::ProperNoun => POSInfo {
            letter: "O",
            _ptb: "NNP",
            _emoji: "📛",
            _name: "proper noun",
        },
        POS::Verb => POSInfo {
            letter: "V",
            _ptb: "VB",
            _emoji: "🏃",
            _name: "verb",
        },
        POS::Adjective => POSInfo {
            letter: "J",
            _ptb: "JJ",
            _emoji: "🌈",
            _name: "adjective",
        },
        POS::Adverb => POSInfo {
            letter: "R",
            _ptb: "RB",
            _emoji: "🤷",
            _name: "adverb",
        },
        POS::Conjunction => POSInfo {
            letter: "C",
            _ptb: "CC",
            _emoji: "🔗",
            _name: "conjunction",
        },
        POS::Determiner => POSInfo {
            letter: "D",
            _ptb: "DT",
            _emoji: "👉",
            _name: "determiner",
        },
        POS::Preposition => POSInfo {
            letter: "P",
            _ptb: "IN",
            _emoji: "📥",
            _name: "preposition",
        },
        POS::Pronoun => POSInfo {
            letter: "I",
            _ptb: "PRP",
            _emoji: "👤",
            _name: "pronoun",
        },
    }
}
