#[derive(Debug, Clone)]
pub enum CliOptionFlags {
    Short(char),
    Long(String),
    ShortAndLong(char, String),
}
impl PartialEq<String> for CliOptionFlags {
    fn eq(&self, other: &String) -> bool {
        match self {
            CliOptionFlags::Short(c) => ("-".to_owned() + &c.to_string()) == *other,
            CliOptionFlags::Long(s) => ("--".to_owned() + s) == *other,
            CliOptionFlags::ShortAndLong(c, s) => {
                ("-".to_owned() + &c.to_string()) == *other || ("--".to_owned() + s) == *other
            }
        }
    }
}
