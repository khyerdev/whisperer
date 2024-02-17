pub struct Recipient {
    ip: String,
    alias: Option<String>
}
impl Recipient {
    pub fn get() -> Vec<Self> {
        todo!()
    }
    pub fn add(addition: impl Into<Recipient>) {
        todo!()
    }
    pub fn set_alias(addition: impl Into<Recipient>) {
        todo!()
    }
    pub fn remove(removal: impl Into<Recipient>) {
        todo!()
    }
    pub fn full_string(&self) -> String {
        match &self.alias {
            None => self.ip.clone(),
            Some(a) => format!("{} ({})", a, &self.ip)
        }
    }
    pub fn alias(&self) -> Option<String> {
        self.alias.clone()
    }
}

impl Into<Recipient> for String {
    fn into(self) -> Recipient {
        Recipient {
            ip: self,
            alias: None
        }
    }
}
impl Into<Recipient> for &str {
    fn into(self) -> Recipient {
        Recipient {
            ip: self.to_string(),
            alias: None
        }
    }
}

pub fn _is_valid_ip(ip: impl ToString) -> bool {
    let ip = ip.to_string();
    let bytes: Vec<&str> = ip.split_terminator('.').collect();
    if bytes.len() != 4 {return false};
    for byte in bytes.iter() {
        match byte.parse::<u8>() {
            Ok(_) => (),
            Err(_) => return false,
        };
    }
    true
}

pub struct Message {
    author: String,
    content: String
}
impl Message {
    pub fn new(author: String, content: String) -> Self {
        Self {author, content}
    }
    pub fn author(&self) -> String {
        self.author.clone()
    }
    pub fn content(&self) -> String {
        self.content.clone()
    }
}