use super::*;

#[derive(Copy, Clone)]
pub enum KeyOrButton {
    Key(geng::Key),
    Mouse(geng::MouseButton),
}

impl Debug for KeyOrButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key) => write!(f, "{key}"),
            Self::Mouse(button) => write!(f, "Mouse{button}"),
        }
    }
}

impl std::fmt::Display for KeyOrButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Failed to parse")]
pub struct ParseError;

impl std::str::FromStr for KeyOrButton {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(key) = s.parse() {
            return Ok(Self::Key(key));
        }
        if let Some(button) = s.strip_prefix("Mouse") {
            if let Ok(button) = button.parse() {
                return Ok(Self::Mouse(button));
            }
        }
        Err(ParseError)
    }
}

impl Serialize for KeyOrButton {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{self:?}").serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KeyOrButton {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = KeyOrButton;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "key or mouse button")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}

pub struct KeyBind {
    pub modifiers: Vec<geng::Key>,
    pub key: KeyOrButton,
}
impl KeyBind {
    pub fn matches(&self, event: &geng::Event, ctx: &Ctx) -> bool {
        for &modifier in &self.modifiers {
            if !ctx.geng.window().is_key_pressed(modifier) {
                return false;
            }
        }
        *event
            == match self.key {
                KeyOrButton::Key(key) => geng::Event::KeyPress { key },
                KeyOrButton::Mouse(button) => geng::Event::MousePress { button },
            }
    }
}

impl std::fmt::Debug for KeyBind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, key) in self
            .modifiers
            .iter()
            .copied()
            .map(KeyOrButton::Key)
            .chain(std::iter::once(self.key))
            .enumerate()
        {
            if i != 0 {
                write!(f, "-")?;
            }
            write!(f, "{key}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for KeyBind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::str::FromStr for KeyBind {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let keys = s.split('-').map(|s| s.parse());
        let mut modifiers = Vec::new();
        let mut bind = None;
        for key in keys {
            if let Some(key) = bind.replace(key?) {
                let KeyOrButton::Key(key) = key else {
                    return Err(ParseError);
                };
                modifiers.push(key);
            }
        }
        Ok(Self {
            modifiers,
            key: bind.ok_or(ParseError)?,
        })
    }
}

impl Serialize for KeyBind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = String::new();
        for (i, k) in self
            .modifiers
            .iter()
            .copied()
            .map(KeyOrButton::Key)
            .chain(std::iter::once(self.key))
            .enumerate()
        {
            if i != 0 {
                s.push('-');
            }
            use std::fmt::Write;
            write!(s, "{k:?}").unwrap();
        }
        s.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for KeyBind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;
        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = KeyBind;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "key or mouse button")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(Visitor)
    }
}
