//! #Speech Engine Information
//!
//! ## Pitch (default 140hz)
//! ### SAPI4: Relative pitch
//! * Number is relative to the default/current pitch.
//! * 50 is 1/2 of the default/current pitch, 200 is 2 times the default/current pitch.
//!
//!  Note: no range is specified by the spec
//! ### SAPI5: Relative pitch
//! * Number is in range -24 to 24
//! * -24 is one octave below the default/current pitch, +24 is one octave above
//! * changes are logarithmic -- a change of +/-1 corresponds to multiplying/dividing by 24th root of 2
//! ### SSML: Relative pitch
//! * pitch in hertz (default/current man's voice is about 100hz, woman's 180hz)
//!
//! Note: other legal values for SSML are not supported, and all numbers are interpreted as relative changes
//! ### Eloquence: Absolute pitch (relative pitch not supported by Eloquence)
//! * Range is 0 - 100.  Guess is that 0 ~= 42hz, 100 ~= 422hz based on supported \"sapi\" values
//! ## Rate (default 180 words/min)
//! ### SAPI4: Absolute rate
//! * Number is relative to the default/current rate
//! * 50 is 1/2 of the default/current rate, 200 is 2 times the default/current rate
//!
//! Note: no range is specified by the spec
//! ### SAPI5: Relative rate
//! * Number is in range -10 to 10
//! * -10 is 1/3 of the default/current speed; 10 3 times the default/current speech
//! * changes are logarithmic -- a change of +/-1 corresponds to multiplying/dividing by 10th root of 3
//! ### SSML: Relative rate
//! * 0.5 is 1/2 of the default/current rate, 2.0 is 2 times the default/current rate
//!
//!  Note:  other legal values for SSML are not supported, and all numbers are interpreted as relative changes
//! ### Eloquence: Absolute rate (relative rate not supported by Eloquence)
//! * Range is 0 - 250, which manual seems to indicate corresponds to 70 - 1297 words/min.
//! * * Window-Eyes only seems to give values in range 1 - 150.
//! * On the low end, 1 ~= 72words/min
//! * On the high end, I can't tell, but 80 seems to be a bit over twice normal (~400 words/min?)
//!    250 ~= 1297 words/min based on supported "sapi" values
//!
//!  Note: this means words/min = 4.18 * Eloquence rate + 66
//!  So the relative pause rate is 180/computed value
//!
//!
//! ## Volume (default 100 \[full])
//! ### SAPI4: Relative volume
//! * Number is relative to the default/current rate
//! * Range is 0 - 065535
//! ### SAPI5: Relative volume
//! * Number is in range 0 to 100
//! ### SSML: Relative volume
//! * Number is in range 0 to 100
//!
//! Note:  other legal values for SSML are not supported, and all numbers are interpreted as relative changes
//! ### Eloquence: Absolute volume (relative volume not supported by Eloquence)
//! * Range is 0 - 100
//!
//! ## Pause
//! * All systems -- pauses are given in milliseconds
//!
//! Note: Pauses on output are scaled based on the ratio of the current rate to the default rate (180 wpm)
#![allow(clippy::needless_return)]

use crate::{errors::*, prefs::PreferenceManager, speech::ReplacementArray};
use sxd_document::dom::Element;
use yaml_rust::Yaml;

use std::{fmt};
use crate::speech::{SpeechRulesWithContext, MyXPath, TreeOrString};
use std::string::ToString;
use std::str::FromStr;
use strum_macros::{Display, EnumString};
use regex::Regex;

const PAUSE_SHORT:f64 = 150.0;  // ms
const PAUSE_MEDIUM:f64 = 300.0; // ms
const PAUSE_LONG:f64 = 600.0;   // ms
const PAUSE_AUTO:f64 = 987654321.5;   // ms -- hopefully unique
pub const PAUSE_AUTO_STR: &str = "\u{F8FA}\u{F8FA}";
const RATE_FROM_CONTEXT:f64 = 987654321.5;   // hopefully unique

/// TTSCommand are the supported TTS commands
/// When parsing the YAML rule files, they are converted to these enums
#[derive(Debug, Clone, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]  // allows lower case
pub enum TTSCommand {
    Pause,
    Rate,
    Volume,
    Pitch ,
    Gender,
    Voice,
    Spell,
    Bookmark,
    Pronounce,
}

#[derive(Debug, Clone)]
pub struct Pronounce {
    text: String,       // plain text
    ipa: String,        // ipa 
    sapi5: String,
    eloquence: String,
}


impl fmt::Display for Pronounce {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut comma = "";     // comma separator so it looks right
        write!(f, "pronounce: [")?;
        if !self.text.is_empty() {
            write!(f, "text: '{}'", self.text)?;
            comma = ",";
        }
        write!(f, "pronounce: [")?;
        if !self.ipa.is_empty() {
            write!(f, "{}ipa: '{}'", comma, self.ipa)?;
            comma = ",";
        }
        write!(f, "pronounce: [")?;
        if !self.sapi5.is_empty() {
            write!(f, "{}sapi5: '{}'", comma, self.sapi5)?;
            comma = ",";
        }
        write!(f, "pronounce: [")?;
        if !self.eloquence.is_empty() {
            write!(f, "{}eloquence: '{}'", comma, self.eloquence)?;
        }
        return writeln!(f, "]");
    }
}

impl Pronounce {
    fn build(values: &Yaml) -> Result<Pronounce> {
        use crate::speech::{as_str_checked, yaml_to_type};
        use crate::pretty_print::yaml_to_string;

        let mut text = "";
        let mut ipa = "";
        let mut sapi5 = "";
        let mut eloquence = "";
        // values should be an array with potential values for Pronounce
        let values = values.as_vec().ok_or_else(||
                                        format!("'pronounce' value '{}' is not an array", yaml_to_type(values)))?;
        for key_value in values {
            let key_value_hash = key_value.as_hash().ok_or_else(|| 
                                        format!("pronounce value '{}' is not key/value pair", yaml_to_string(key_value, 0)))?;
            if key_value_hash.len() != 1 {
                bail!("pronounce value {:?} is not a single key/value pair", key_value_hash);
            }
        
            for (key, value) in key_value_hash {
                match as_str_checked(key)? {
                    "text" => text = as_str_checked(value)?,
                    "ipa" => ipa = as_str_checked(value)?,
                    "sapi5" => sapi5 = as_str_checked(value)?,
                    "eloquence" => eloquence = as_str_checked(value)?,
                    _ => bail!("unknown pronounce type: {} with value {}", yaml_to_string(key, 0), yaml_to_string(value, 0)),
                }
            }
        }
        if text.is_empty() {
            bail!("'text' key/value is required for 'pronounce' -- it is used is the speech engine is unknown.")
        }
        return Ok( Pronounce{
            text: text.to_string(),
            ipa: ipa.to_string(),
            sapi5: sapi5.to_string(),
            eloquence: eloquence.to_string()
        } );
    

    }
}
/// TTSCommands are either numbers (f64 because of YAML) or strings
#[derive(Debug, Clone)]
pub enum TTSCommandValue {
    Number(f64),
    String(String),
    XPath(MyXPath),
    Pronounce(Box<Pronounce>),
}

impl TTSCommandValue {
    fn get_num(&self) -> f64 {
        match self {
            TTSCommandValue::Number(n) => return *n,
            _                               => panic!("Internal error: TTSCommandValue is not a number"),
        }
    }

    fn get_string(&self) -> &String {
        match self {
            TTSCommandValue::String(s) => return s,
            _                                  => panic!("Internal error: TTSCommandValue is not a string"),
        }
    }

    fn get_pronounce(&self) -> &Pronounce {
        match self {
            TTSCommandValue::Pronounce(p) => return p,
            _                               => panic!("Internal error: TTSCommandValue is not a 'pronounce' command'"),
        }
        
    }
}

/// A TTS rule consists of the command, the value, and its replacement
#[derive(Debug, Clone)]
pub struct TTSCommandRule {
    command: TTSCommand,
    value: TTSCommandValue,
    replacements: ReplacementArray
}

impl fmt::Display for TTSCommandRule {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let value = match &self.value {
            TTSCommandValue::String(s) => s.to_string(),
            TTSCommandValue::Number(f) => f.to_string(),
            TTSCommandValue::XPath(p) => p.to_string(),
            TTSCommandValue::Pronounce(p) => p.to_string(),
        };
        if self.command == TTSCommand::Pause {
            return write!(f, "pause: {}", value);
        } else {
            return write!(f, "{}: {}{}", self.command, value, self.replacements);
        };
    }
}


impl TTSCommandRule {
    pub fn new(command: TTSCommand, value: TTSCommandValue, replacements: ReplacementArray) -> TTSCommandRule {
        return TTSCommandRule{
            command,
            value,
            replacements
        }
    }
}

/// Supported TTS engines
/// These types should do something for all the TTSCommands
#[allow(clippy::upper_case_acronyms)]
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub enum TTS {
    None,
    SAPI5,
    SSML,
//    Mac,
//    Eloquence
}

impl TTS {
    /// Given the tts command ("pause", "rate", etc) and its value, build the TTS data structure for it.
    ///
    /// `tts_command`: one of "pause", "rate", etc
    ///
    /// `value`: keyword 'value' or dict with 'value' and 'replace' (optional) keys
    pub fn build(tts_command: &str, values: &Yaml) -> Result<Box<TTSCommandRule>> {
        use crate::pretty_print::yaml_to_string;
        let hashmap = values.as_hash();
        let tts_value;
        let replacements;
        if hashmap.is_some() {
            tts_value = &values["value"];
            if tts_value.is_badvalue() {
                bail!("{} TTS command is missing a 'value' sub-key. Found\n{}", tts_command, yaml_to_string(values, 1));
            };
            replacements = ReplacementArray::build(&values["replace"])?;
        } else {
            tts_value = values;
            replacements = ReplacementArray::build_empty();
        }
        let tts_value = yaml_to_string(tts_value, 0);
        let tts_value = tts_value.trim(); // if not a number or string, value is bogus
        let tts_enum = match TTSCommand::from_str(tts_command) {
            Ok(t) => t,
            Err(_) => bail!("Internal error in build_tts: unexpected rule ({:?}) encountered", tts_command),
        };
    
        let tts_command_value;
        if tts_enum == TTSCommand::Pause || tts_enum == TTSCommand::Rate || tts_enum == TTSCommand::Volume || tts_enum == TTSCommand::Pitch {
            let val = match tts_value {
                "short" => Ok( PAUSE_SHORT ),
                "medium" => Ok( PAUSE_MEDIUM ),
                "long" => Ok( PAUSE_LONG ),
                "auto" => Ok( PAUSE_AUTO ),
                "$MathRate" => Ok( RATE_FROM_CONTEXT ), // special case hack -- value determined in replace
                _ => tts_value.parse::<f64>()
            };
        
            if val.is_err() {
                bail!(format!("\"{}: {}\" is not a number", &tts_command, tts_value));    
            }
    
            tts_command_value = TTSCommandValue::Number(val.unwrap());
        } else if tts_enum == TTSCommand::Bookmark {
            tts_command_value = TTSCommandValue::XPath(
                MyXPath::build(values).chain_err(|| "while trying to evaluate value of 'bookmark:'")?
            );
        } else if tts_enum == TTSCommand::Pronounce {
            tts_command_value = TTSCommandValue::Pronounce( Box::new( Pronounce::build(values)? ) );
        } else {
            tts_command_value = TTSCommandValue::String(tts_value.to_string());
        }
        return Ok( Box::new( TTSCommandRule::new(tts_enum, tts_command_value, replacements) ) );
    }
    
    /// The rule called to execute the TTSCommand `command`
    /// `prefs` are used for scaling the speech rate
    /// some rules have MathML nested inside, so we need to do replacements on them (hence `rules` and `mathml` are needed)
    ///
    /// A string is returned for the speech engine.
    ///
    /// `auto` pausing is handled at a later phase and a special char is used for it
    pub fn replace<'c, 's:'c, 'm:'c, 'r, T:TreeOrString<'c, 'm, T>>(&self, command: &TTSCommandRule, prefs: &PreferenceManager, rules_with_context: &'r mut SpeechRulesWithContext<'c, 's, 'm>, mathml: Element<'c>) -> Result<T> {
        return T::replace_tts(self, command, prefs, rules_with_context, mathml);
    }

    pub fn replace_string<'c, 's:'c, 'm, 'r>(&self, command: &TTSCommandRule, prefs: &PreferenceManager, rules_with_context: &'r mut SpeechRulesWithContext<'c, 's, 'm>, mathml: Element<'c>) -> Result<String> {
        // The general idea is we handle the begin tag, the contents, and then the end tag
        // For the begin/end tag, we dispatch off to specialized code for each TTS engine

        // bookmarks are special in that we need to eval the xpath
        // rather than pass a bunch of extra info into the generic handling routines, we just deal with them here
        if command.command == TTSCommand::Bookmark {
           // if we aren't suppose to generate bookmarks, short circuit and just return
           if prefs.get_api_prefs().to_string("Bookmark") != "true"{
            return Ok("".to_string());
           }
           return Ok( match self {
                TTS::None  => "".to_string(),
                TTS::SAPI5 => compute_bookmark_element(&command.value, "bookmark mark", rules_with_context, mathml)?,
                TTS::SSML => compute_bookmark_element(&command.value, "mark name", rules_with_context, mathml)?,
            } );
        }

        let mut command = command.clone();
        if command.command == TTSCommand::Rate && self != &TTS::None {
            if let TTSCommandValue::Number(number_value) = command.value {
                if number_value == RATE_FROM_CONTEXT {
                    // handle hack for $Rate -- need to look up in context
                    let rate_from_context = crate::navigate::context_get_variable(rules_with_context.get_context(), "MathRate", mathml)?.1;
                    assert!(rate_from_context.is_some());
                    command.value = TTSCommandValue::Number(rate_from_context.unwrap());
                }
            }
        }


        let mut result = String::with_capacity(255);
        result += &match self {
            TTS::None  => self.get_string_none(&command, prefs, true),
            TTS::SAPI5 => self.get_string_sapi5(&command, prefs, true),
            TTS::SSML  => self.get_string_ssml(&command, prefs, true),
        };


        if !command.replacements.is_empty()  {
            if result.is_empty() {
                result += " ";
            }
            result += &command.replacements.replace::<String>(rules_with_context, mathml)?;    
        }

        let end_tag = match self {
            TTS::None  => self.get_string_none(&command, prefs, false),
            TTS::SAPI5 => self.get_string_sapi5(&command, prefs, false),
            TTS::SSML  => self.get_string_ssml(&command, prefs, false),
        };

        if end_tag.is_empty() {
            return Ok( result ); // avoids adding in " "
        } else {
            return Ok( result + &end_tag );
        }


        fn compute_bookmark_element<'c, 's:'c, 'm, 'r>(value: &TTSCommandValue, tag_and_attr: &str, rules_with_context: &'r mut SpeechRulesWithContext<'c, 's, 'm>, mathml: Element<'c>) -> Result<String> {
            match value {
                TTSCommandValue::XPath(xpath) => {
                    let id = xpath.replace::<String>(rules_with_context, mathml)?;
                    return Ok( format!("<{}='{}'/>", tag_and_attr, id) );
                },
                _ => bail!("Implementation error: found non-xpath value for bookmark"),
            }
        }
    
    }

    // auto pausing can't be known until neighboring strings are computed
    // we create a unique string in this case and compute the real value later 
    fn get_string_none(&self, command: &TTSCommandRule,  _prefs: &PreferenceManager, is_start_tag: bool) -> String  {
        // they only thing to do is handle "pause" with some punctuation hacks along with 'spell'        
        if is_start_tag {
            if command.command == TTSCommand::Pause {
                let amount = command.value.get_num();
                // only ',' and ';' are used as '.' didn't seem to reliably generate pauses in tests
                return crate::speech::CONCAT_INDICATOR.to_string() + (
                    if amount == PAUSE_AUTO {
                        PAUSE_AUTO_STR
                    } else if amount <= 250.0 {
                        ","
                    } else  {
                        ";"
                    }
                );
            } else if command.command == TTSCommand::Spell {
                // add a space between the chars
                let mut chars = command.value.get_string().chars();
                let first_char = chars.next();
                if first_char.is_none() {
                    return "".to_string();
                }
            
                let mut chars_with_spaces = vec![first_char.unwrap()];
                for ch in chars {
                    chars_with_spaces.push(' ');
                    chars_with_spaces.push(ch);
                }
                return chars_with_spaces.into_iter().collect();
            } else if let TTSCommandValue::Pronounce(p) = &command.value {
                return crate::speech::CONCAT_INDICATOR.to_string() + &p.text;
            }
        };
        return "".to_string();
    }
    
    fn get_string_sapi5(&self, command: &TTSCommandRule, prefs: &PreferenceManager, is_start_tag: bool) -> String  {
        return match &command.command {
            TTSCommand::Pause => if is_start_tag {
                let amount = command.value.get_num();
                if amount == PAUSE_AUTO {
                    PAUSE_AUTO_STR.to_string()
                } else {
                    format!("<silence msec=='{}ms'/>", amount * 180.0/prefs.get_rate())
                }
            } else {
                "".to_string()
            },
            TTSCommand::Pitch => if is_start_tag {format!("<pitch middle='XXX{}%'>", command.value.get_num())} else {String::from("</prosody>")},
            TTSCommand::Rate =>  if is_start_tag {format!("<rate speed='XXX{}%'>", command.value.get_num())} else {String::from("</rate>")},
            TTSCommand::Volume =>if is_start_tag {format!("<volume level='{}'>", command.value.get_num())} else {String::from("</volume>")},
            TTSCommand::Gender =>if is_start_tag {format!("<prosody gender='XXX{}%'>", command.value.get_string())} else {String::from("</prosody>")},
            TTSCommand::Voice =>if is_start_tag {format!("<prosody voice='XXX{}%'>", command.value.get_string())} else {String::from("</prosody>")},
            TTSCommand::Spell =>if is_start_tag {format!("<spell>{}", command.value.get_string())} else {String::from("</spell>")},
            TTSCommand::Pronounce =>if is_start_tag {
                    format!("<pron sym='{}'>{}", &command.value.get_pronounce().sapi5, &command.value.get_pronounce().text)
                } else {
                    String::from("</pron>")
                },
            TTSCommand::Bookmark => panic!("Internal error: bookmarks should have been handled earlier"),
        };
    }

    fn get_string_ssml(&self, command: &TTSCommandRule, prefs: &PreferenceManager, is_start_tag: bool) -> String  {
        return match &command.command {
            TTSCommand::Pause => {
                if is_start_tag {
                    let amount = command.value.get_num();
                    if amount == PAUSE_AUTO {
                        PAUSE_AUTO_STR.to_string()
                    } else {
                        format!("<break time='{}ms'/>", amount * 180.0/prefs.get_rate())
                    }
                } else {
                    "".to_string()
                }
            },
            TTSCommand::Pitch => if is_start_tag {format!("<prosody pitch='{}%'>", command.value.get_num())} else {String::from("</prosody>")},
            TTSCommand::Rate =>  if is_start_tag {format!("<prosody rate='{}%'>", command.value.get_num())} else {String::from("</prosody>")},
            TTSCommand::Volume =>if is_start_tag {format!("<prosody volume='{}%'>", command.value.get_num())} else {String::from("</prosody>")},
            TTSCommand::Gender =>if is_start_tag {format!("<voice required='gender=\"{}\"'>", command.value.get_string())} else {String::from("</voice>")},
            TTSCommand::Voice =>if is_start_tag {format!("<voice required='{}'>", command.value.get_string())} else {String::from("</voice>")},
            TTSCommand::Spell =>if is_start_tag {format!("<say-as interpret-as='characters'>{}", command.value.get_string())} else {String::from("</say-as>")},
            TTSCommand::Pronounce =>if is_start_tag {
                format!("<phoneme alphabet='ipa' ph='{}'>{}", &command.value.get_pronounce().ipa, &command.value.get_pronounce().text)
            } else {
                String::from("</phoneme>")
            },
        TTSCommand::Bookmark => panic!("Internal error: bookmarks should have been handled earlier"),
        }
    }

    /// Compute the length of the pause to use.
    ///
    /// The computation is based on the length of the speech strings (after removing tagging).
    /// There is a bias towards pausing more _after_ longer strings.
    pub fn compute_auto_pause(&self, prefs: &PreferenceManager, before: &str, after: &str) -> String {
        lazy_static! {
            static ref REMOVE_XML: Regex = Regex::new(r"<.+?>").unwrap();    // punctuation ending with a '.'
        }
        let before_len;
        let after_len;
        match self {
            TTS::SSML | TTS::SAPI5 => {
                before_len = REMOVE_XML.replace_all(before, "").len();
                after_len = REMOVE_XML.replace_all(after, "").len();
            },
            _ => {
                before_len = before.len();
                after_len = after.len();
            },
        }

        // pause values are not cut in stone
        // the calculation bias to 'previous' is based on MathPlayer which used '30 * #-of-descendants-on-left
        // I think I did this as a sort of "take a breath" after saying something long although one might want to do that
        //   before speaking something long.
        if after_len < 3 {
            // hack to prevent pausing before "of" in exprs like "the fourth power of secant, of x"
            // if it should pause anywhere, it should be after the "of"
            return "".to_string(); 
        }
        let pause = std::cmp::min(3000, ((2 * before_len + after_len)/48) * 128);
        if pause <= 50 {
            // don't put out a lot of short pauses which probably can't be heard
            return "".to_string();
        }
        // create a TTSCommandRule so we reuse code
        let command = TTSCommandRule::new(
            TTSCommand::Pause,
            TTSCommandValue::Number(pause as f64),
            ReplacementArray::build_empty(),
        );
        return match self {
            TTS::None  => self.get_string_none(&command, prefs, true),
            TTS::SAPI5 => self.get_string_sapi5(&command, prefs, true),
            TTS::SSML  => self.get_string_ssml(&command, prefs, true),
        };

    }

    /// Take the longest of the pauses
    ///
    /// Two other options are:
    /// 1. average the pauses
    /// 2. add the pauses together.
    ///
    /// Until evidence points otherwise, use 'longest'.
    pub fn merge_pauses(&self, str: &str) -> String {
        // we need specialized merges for each TTS engine because we need to know the format of the commands
        return match self {
            TTS::None  => self.merge_pauses_none(str),
            TTS::SAPI5 => self.merge_pauses_sapi5(str),
            TTS::SSML  => self.merge_pauses_ssml(str),
        };        
    }

    fn merge_pauses_none(&self, str: &str) -> String {
        // punctuation used for pauses is ",", ";" 
        lazy_static! {
            static ref MULTIPLE_PAUSES: Regex = Regex::new(r"[,;][,;]+").unwrap();   // two or more pauses
        }
        // we reduce all sequences of two or more pauses to a single medium pause
        let mut merges_string = str.to_string();
        for cap in MULTIPLE_PAUSES.captures_iter(str) {
            merges_string = merges_string.replace(&cap[0], ";");
        }
        return merges_string;
    }

    fn merge_pauses_xml<F>(str: &str, full_attr_re: &Regex, sub_attr_re: &Regex, replace_with: F) -> String 
            where F: Fn(usize) -> String {
        // we reduce all sequences of two or more pauses to the max pause amount
        // other options would be the sum or an average
        // maybe some amount a little longer than the max would be best???
        let mut merges_string = str.to_string();
        for cap in full_attr_re.captures_iter(str) {
            let mut amount = 0;
            for c in sub_attr_re.captures_iter(&cap[0]) {
                amount = std::cmp::max(amount, c[1].parse::<usize>().unwrap());
            };
            merges_string = merges_string.replace(&cap[0], &replace_with(amount));
        }
        return merges_string;
    }

    fn merge_pauses_sapi5(&self, str: &str) -> String {
        lazy_static! {
            static ref CONSECUTIVE_BREAKS: Regex = Regex::new(r"(<silence msec[^>]+?> *){2,}").unwrap();   // two or more pauses
            static ref PAUSE_AMOUNT: Regex = Regex::new(r"msec=.*?(\d+)").unwrap();   // amount after 'time'
        }
        let replacement = |amount: usize| format!("<silence msec=='{}ms'/>", amount);
        return TTS::merge_pauses_xml(str, &CONSECUTIVE_BREAKS, &PAUSE_AMOUNT, replacement);
    }

    fn merge_pauses_ssml(&self, str: &str) -> String {
        lazy_static! {
            static ref CONSECUTIVE_BREAKS: Regex = Regex::new(r"(<break time=[^>]+?> *){2,}").unwrap();   // two or more pauses
            static ref PAUSE_AMOUNT: Regex = Regex::new(r"time=.*?(\d+)").unwrap();   // amount after 'time'
        }
        let replacement = |amount: usize| format!("<break time='{}ms'/>", amount);
        return TTS::merge_pauses_xml(str, &CONSECUTIVE_BREAKS, &PAUSE_AMOUNT, replacement);
    }
}