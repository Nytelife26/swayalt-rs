use std::fmt::{self, Display};

use swayipc::{Connection, Event, EventType, Floating, Rect, WindowChange};

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Split {
	H,
	V,
}

impl From<Rect> for Split {
	fn from(value: Rect) -> Self {
		// SAFETY: bool will be 0 or 1, which are the only enum variants.
		unsafe { std::mem::transmute(value.height > value.width) }
	}
}

impl Display for Split {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Split::H => "splith",
				Split::V => "splitv",
			}
		)
	}
}

#[inline(always)]
pub fn is_tiling(floating: Option<Floating>) -> bool {
	matches!(floating, Some(Floating::AutoOff | Floating::UserOff))
}

#[inline(always)]
pub fn try_set_split(
	conn: &mut Connection,
	id: i64,
	split: Split,
) -> Result<Vec<Result<(), swayipc::Error>>, swayipc::Error> {
	conn.run_command(format!("[con_id=\"{id}\"] {split}"))
}

fn main() -> anyhow::Result<()> {
	let mut conn = Connection::new()?;
	let mut events = Connection::new()?.subscribe([EventType::Window])?;
	while let Some(Ok(Event::Window(window))) = events.next() {
		let focused_node = window.container;
		if window.change == WindowChange::Focus
			&& is_tiling(focused_node.floating)
		{
			try_set_split(
				&mut conn,
				focused_node.id,
				focused_node.rect.into(),
			)?;
		}
	}
	Ok(())
}
