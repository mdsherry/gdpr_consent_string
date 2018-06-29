#[macro_use]
extern crate bitmask;
extern crate chrono;

use chrono::{DateTime, TimeZone, NaiveDateTime, Utc};
mod consts;

bitmask! {
    pub mask Purposes: u32 where flags Purpose {
        /// Storage and access of information
        ///
        /// The storage of information, or access to information that is already stored, on user device such as
        /// accessing advertising identifiers and/or other device identifiers, and/or using cookies or similar
        /// technologies.
        StorageAndAccess = 1,

        /// Personalisation
        ///
        /// The collection and processing of information about user of a site to subsequently personalize
        /// advertising for them in other contexts, i.e. on other sites or apps, over time. Typically, the content
        /// of the site or app is used to make inferences about user interests, which inform future selections.
        Personalization = 2,

        /// Ad selection, reporting and delivery
        ///
        /// The collection of information and combination with previously collected information, to select and 
        /// deliver advertisements and to measure the delivery and effectiveness of such advertisements. This includes
        /// using previously collected information about user interests to select ads, processing data about what
        /// advertisements were shown, how often they were shown, when and where they were shown, and whether they 
        /// took any action related to the advertisement, including for example clicking an ad or making a purchase.
        AdSelection = 4,

        /// Content delivery, selection and reporting
        ///
        /// The collection of information, and combination with previously collected information, to select and deliver
        /// content and to measure the delivery and effectiveness of such content. This includes using previously
        /// collected information about user interests to select content, processing data about what content was shown,
        /// how often or how long it was shown, when and where it was shown, and whether they took any action related
        /// to the content, including for example clicking on content.
        ContentDelivery = 8,

        ///Measurement
        ///
        /// The collection of information about user use of content, and combination with previously collected
        /// information, used to measure, understand, and report on user usage of content.
        Measurement = 16
    }
}

impl Purposes {
    fn from_raw(raw: u32) -> Purposes {
        Purposes { mask: raw }
    }
}

fn decode(c: char) -> u8 {
    match c {
        'A'...'Z' => c as u8 - 'A' as u8,
        'a'...'z' => c as u8 - 'a' as u8 + 26,
        '0'...'9' => c as u8 - '0' as u8 + 52,
        '-' => 62,
        '_' => 63,
        _ => panic!("Character {} is not a valid Base64 character", c),
    }
}

fn take_6<T: Iterator<Item = char>>(it: &mut T) -> Option<u8> {
    it.next().map(decode)
}

fn take_12<T: Iterator<Item = char>>(it: &mut T) -> Option<u16> {
    let a = it.next().map(decode)?;
    let b = it.next().map(decode)?;
    Some((a as u16) << 6 | (b as u16))
}

fn take_4<T: Iterator<Item = char>>(it: &mut T) -> Option<u32> {
    let a = it.next().map(decode)?;
    let b = it.next().map(decode)?;
    let c = it.next().map(decode)?;
    let d = it.next().map(decode)?;

    Some((a as u32) << 18 | (b as u32) << 12 | (c as u32) << 6 | (d as u32))
}

fn take_36<T: Iterator<Item = char>>(it: &mut T) -> Option<u64> {
    let a = it.next().map(decode)?;
    let b = it.next().map(decode)?;
    let c = it.next().map(decode)?;
    let d = it.next().map(decode)?;
    let e = it.next().map(decode)?;
    let f = it.next().map(decode)?;

    Some(
        (a as u64) << 30 | (b as u64) << 24 | (c as u64) << 18 | (d as u64) << 12 | (e as u64) << 6
            | (f as u64),
    )
}

fn language_code<T: Iterator<Item = char>>(it: &mut T) -> Option<[char; 2]> {
    use consts::LETTERS;
    let a = it.next().map(decode)?;
    let b = it.next().map(decode)?;
    Some([LETTERS[a as usize], LETTERS[b as usize]])
}

fn purpose<T: Iterator<Item = char>>(it: &mut T) -> Option<u32> {
    use consts::REVERSE_BITS;
    let a = it.next().map(decode)?;
    let b = it.next().map(decode)?;
    let c = it.next().map(decode)?;
    let d = it.next().map(decode)?;
    Some(
        (REVERSE_BITS[d as usize] as u32) << 18 | (REVERSE_BITS[c as usize] as u32) << 12
            | (REVERSE_BITS[b as usize] as u32) << 6 | (REVERSE_BITS[a as usize] as u32),
    )
}

#[derive(Debug)]
pub struct ConsentString {
    pub version: u8,
    pub created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub cmp_id: u16,
    pub cmp_version: u16,
    pub consent_screen: u8,
    pub consent_language: [char; 2],
    pub vendor_list_version: u16,
    pub purposes_allowed: Purposes,
    pub max_vendor_id: u16,
    pub vendor_consents: Vec<bool>
}

#[derive(Debug)]
pub(crate) struct BitDecoder<T> {
    base: T,
    offset: u8,
    leftover: u8
}

impl<T: Iterator<Item=char> + std::fmt::Debug> BitDecoder<T> {
    pub fn new(base: T) -> Self {
        BitDecoder { base: base.into_iter(), offset: 0, leftover: 0 }
    }
    pub fn take(&mut self, n: u8) -> Option<usize> {
        if self.offset == 0 {
            self.leftover = decode(self.base.next()?);
            self.offset = 6;
        }
        let mask = (1 << self.offset) - 1;
        if self.offset >= n {
            // We have enough left to provide this request
            let rv = (self.leftover & mask) >> (self.offset - n);
            self.offset -= n;

            Some(rv as usize)
        } else {
            // We don't have enough. Take what we can first, and add the rest
            let missing = n - self.offset;
            let rv = (self.leftover & mask) as usize;
            self.offset = 0;
            Some(rv << missing | self.take(missing)?)
        }
    }
    pub fn take_bool(&mut self) -> Option<bool> {
        Some(self.take(1)? == 1)
    }
}

impl ConsentString {
    pub fn parse(str: &str) -> Option<ConsentString> {
        let mut chars = str.chars();
        let version = take_6(&mut chars)?;
        let created = take_36(&mut chars)?;
        let created = NaiveDateTime::from_timestamp((created / 10) as i64, ((created % 10) * 100_000_000) as u32);
        let created = DateTime::<Utc>::from_utc(created, Utc);
        let last_updated = take_36(&mut chars)?;
        let last_updated = NaiveDateTime::from_timestamp((last_updated / 10) as i64, ((last_updated % 10) * 100_000_000) as u32);
        let last_updated = DateTime::<Utc>::from_utc(last_updated, Utc);
        let cmp_id = take_12(&mut chars)?;
        let cmp_version = take_12(&mut chars)?;
        let consent_screen = take_6(&mut chars)?;
        let consent_language = language_code(&mut chars)?;
        let vendor_list_version = take_12(&mut chars)?;
        let purposes_allowed = Purposes::from_raw(purpose(&mut chars)?);
        let mut bd = BitDecoder::new(chars);
        let max_vendor_id = bd.take(16)? as u16;
        
        let range_encoding = bd.take_bool()?;
        let vendor_consents: Vec<bool> = if range_encoding {
            let default_consent = bd.take_bool()?;
            let mut consents = vec![default_consent; max_vendor_id as usize];
            let num_entries = bd.take(12)?;
            for _ in 0..num_entries {
                let range = bd.take_bool()?;
                if range {
                    let start_vendor_id = bd.take(16)? as usize;
                    let end_vendor_id = bd.take(16)? as usize;
                    for vendor_id in start_vendor_id..=end_vendor_id {
                        consents[vendor_id] = !default_consent;    
                    }
                } else {
                    let vendor_id = bd.take(16)? as usize;
                    consents[vendor_id] = !default_consent;
                }
            }
            consents
        } else {
            let mut rv = Vec::with_capacity(max_vendor_id as usize);
            for _ in 0..max_vendor_id {
                rv.push(bd.take_bool()?);
            }
            rv
        };

        // chars.map(decode).for_each(|b| print!("{:06b}", b));
        Some(ConsentString {
            version,
            created,
            last_updated,
            cmp_id,
            cmp_version,
            consent_screen,
            consent_language,
            vendor_list_version,
            purposes_allowed,
            max_vendor_id,
            vendor_consents
        })
    }
}

#[cfg(test)]
mod tests {
    use *;
    #[test]
    fn it_works() {
        let input = "BOEFEAyOEFEAyAHABDENAI4AAAB9vABAASA";
        let consent_string = ConsentString::parse(input).unwrap();

        assert_eq!(consent_string.version, 1);
        let expected_time = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1510082155, 400_000_000), Utc);
        assert_eq!(consent_string.created, expected_time);
        assert_eq!(consent_string.last_updated, expected_time);
        assert_eq!(consent_string.cmp_id, 7);
        assert_eq!(consent_string.cmp_version, 1);
        assert_eq!(consent_string.consent_screen, 3);
        assert_eq!(consent_string.consent_language, ['e', 'n']);
        assert_eq!(consent_string.vendor_list_version, 8);
        assert_eq!(consent_string.purposes_allowed, Purpose::StorageAndAccess | Purpose::Personalization | Purpose::AdSelection);
        assert_eq!(consent_string.max_vendor_id, 2011);
        let mut consents = vec![true; 2011];
        consents[9] = false;
        assert_eq!(consent_string.vendor_consents, consents);
    }

    #[test]
    fn thingie_iter() {
        let words: Vec<char> = vec!['c', 'c'];
        let mut bc = BitDecoder::new(words.into_iter());
        assert_eq!(Some(3), bc.take(3));
        assert_eq!(Some(8), bc.take(4));
        assert_eq!(Some(7), bc.take(3));
        assert_eq!(None, bc.take(3));
    }
}
