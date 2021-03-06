use super::Result;
use super::error::ASN1Error;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]

/// Enum containing all UniversalTypes' Numbers
pub enum UniversalTypes
{
    Eoc              = 0,
    Boolean          = 1,
    Integer          = 2,
    BitString        = 3,
    OctetString      = 4,
    Null             = 5,
    ObjectIdentifier = 6,
    ObjectDescriptor = 7,
    External         = 8,
    Real             = 9,
    Enumerated       = 10,
    EmbeddedPdv      = 11,
    Utf8String       = 12,
    RelativeOid      = 13,
    Sequence         = 16,
    Set              = 17,
    NumericString    = 18,
    PrintableString  = 19,
    T61String        = 20,
    VideotexString   = 21,
    Ia5String        = 22,
    UtcTime          = 23,
    GeneralizedTime  = 24,
    GraphicString    = 25,
    VisibleString    = 26,
    GeneralString    = 27,
    UniversalString  = 28,
    CharacterString  = 29,
    BmpString        = 30,
}

impl UniversalTypes
{
    pub fn from_u8(v: u8) -> Result<UniversalTypes>
    {
        match v
        {
            0 =>  Ok(UniversalTypes::Eoc),
            1 =>  Ok(UniversalTypes::Boolean),
            2 =>  Ok(UniversalTypes::Integer),
            3 =>  Ok(UniversalTypes::BitString),
            4 =>  Ok(UniversalTypes::OctetString),
            5 =>  Ok(UniversalTypes::Null),
            6 =>  Ok(UniversalTypes::ObjectIdentifier),
            7 =>  Ok(UniversalTypes::ObjectDescriptor),
            8 =>  Ok(UniversalTypes::External),
            9 =>  Ok(UniversalTypes::Real),
            10 => Ok(UniversalTypes::Enumerated),
            11 => Ok(UniversalTypes::EmbeddedPdv),
            12 => Ok(UniversalTypes::Utf8String),
            13 => Ok(UniversalTypes::RelativeOid),
            16 => Ok(UniversalTypes::Sequence),
            17 => Ok(UniversalTypes::Set),
            18 => Ok(UniversalTypes::NumericString),
            19 => Ok(UniversalTypes::PrintableString),
            20 => Ok(UniversalTypes::T61String),
            21 => Ok(UniversalTypes::VideotexString),
            22 => Ok(UniversalTypes::Ia5String),
            23 => Ok(UniversalTypes::UtcTime),
            24 => Ok(UniversalTypes::GeneralizedTime),
            25 => Ok(UniversalTypes::GraphicString),
            26 => Ok(UniversalTypes::VisibleString),
            27 => Ok(UniversalTypes::GeneralString),
            28 => Ok(UniversalTypes::UniversalString),
            29 => Ok(UniversalTypes::CharacterString),
            30 => Ok(UniversalTypes::BmpString),
            // BER uses 5 bits to encode the universal tags, and 31/0x1F/b11111 is used
            // to signal to use the long form of encoding
            // FIXME: This is a public function, better error handling!
            _  => unreachable!(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Class
{
    // TODO: Use BigInt instead of i64 to make encoding arbitrary sizes possbile?
    // Much more TODO: Find out if even necessary for LDAP
    Universal(UniversalTypes),
    Application(i64),
    ContextSpecific(i64),
    Private(i64),
}

impl Class
{
    pub fn construct(class: u8, number: i64) -> Result<Class>
    {
        match class
        {
            0 => Ok(Class::Universal(try!(UniversalTypes::from_u8(number as u8)))),
            1 => Ok(Class::Application(number)),
            2 => Ok(Class::ContextSpecific(number)),
            3 => Ok(Class::Private(number)),
            // TODO: Add a more specific error for this.
            _ => Err(ASN1Error::InvalidASN1)
        }
    }
}

/// ClassNumber contains the numerical representation of the tags class
#[derive(Debug, Copy, Clone)]
pub enum ClassNumber
{
    Universal       = 0,
    Application     = 1,
    ContextSpecific = 2,
    Private         = 3,
}

impl ClassNumber
{
    pub fn from_u8(v: u8) -> ClassNumber
    {
        match v
        {
            0 => ClassNumber::Universal,
            1 => ClassNumber::Application,
            2 => ClassNumber::ContextSpecific,
            3 => ClassNumber::Private,
            // FIXME: Better Error handling
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Structure
{
    Primitive   = 0,
    Constructed = 1,
}

impl Structure
{
    pub fn from_u8(v: u8) -> Structure
    {
        match v
        {
            0 => Structure::Primitive,
            1 => Structure::Constructed,
            // FIXME: Better Error handling
            _ => unreachable!(),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub struct Type
{
    pub class: Class,
    pub structure: Structure
}

#[derive(PartialEq, Eq, Debug)]
pub enum Payload
{
    Primitive(Vec<u8>),
    Constructed(Vec<Tag>),
}

impl Payload
{
    pub fn len(&self) -> u64
    {
        let mut l: u64 = 0;
        match *self
        {
            Payload::Primitive(ref v) => l = v.len() as u64,
            Payload::Constructed(ref v) =>
            {
                for tag in v
                {
                    l += tag.size as u64;
                }
            }
        }

        return l;

    }

}

#[derive(PartialEq, Eq, Debug)]
pub struct Tag
{
    pub _type: Type,
    // Encoded length
    pub _length: u64,
    pub _value: Payload,

    // Lenght of the tag including type and length bytes
    pub size: u64,
}

pub fn construct(class: Class, payload: Payload) -> Tag
{
    let tagtype = Type
    {
        class: class,
        structure: match payload
        {
            Payload::Primitive(_) => Structure::Primitive,
            Payload::Constructed(_) => Structure::Constructed,
        },
    };

    let pllen = payload.len();

    let taglen = calculate_len(&tagtype, &pllen);

    Tag
    {
        _type: tagtype,
        _value: payload,
        _length: pllen,
        size: taglen,
    }
}

// we use the given length and do not calculate the length from the payload
pub fn calculate_len(tagtype: &Type, pllen: &u64) -> u64
{
    let mut length: u64 = 0;

    // Get the Lenght of the Class/PC/Type byte(s)
    length += match tagtype.class
    {
        Class::Universal(_) => /* Universal is always exactly one byte */ 1,
        Class::Application(tag) | Class::ContextSpecific(tag) | Class::Private(tag) =>
        {
            // In case of the other three we actually have to look at their content
            let mut len = 1u64;
            if tag > 127
            {
                let mut tag = tag;
                while
                {
                    len += 1;
                    tag >>= 7;
                    tag > 0
                } {}
            }
            len
        }
    };

    // Add space the length bytes take up
    if *pllen < 128
    {
        // Short form was used -> Just one byte
        length += 1;
    }
    else
    {
        let mut len = *pllen;
        while
        {
            len >>= 8;
            length += 1;

            len > 0
        } {}

        length += 1;
    }

    // Add payload length
    length += *pllen as u64;

    length
}
