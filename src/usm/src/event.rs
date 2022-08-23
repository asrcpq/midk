use crate::content::UmeContent;

#[derive(Clone, Debug)]
pub struct UmeEvent {
	pub dt: u32,
	pub mt: u32,
	pub content: UmeContent,
}
