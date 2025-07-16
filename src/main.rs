use std::{
	fmt::{self, Display},
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering},
	},
};

use swayipc::{
	Connection,
	Event,
	EventType,
	Fallible,
	Floating,
	Rect,
	WindowChange,
};

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
		write!(f, "{}", match self {
			Split::H => "splith",
			Split::V => "splitv",
		})
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
) -> Fallible<Vec<Fallible<()>>> {
	conn.run_command(format!("[con_id=\"{id}\"] {split}"))
}

fn main() -> anyhow::Result<()> {
	let running = Arc::new(AtomicBool::from(true));
	let r = running.clone();
	ctrlc::set_handler(move || {
		r.store(false, Ordering::Relaxed);
	})?;

	let mut conn = Connection::new()?;
	let mut events = Connection::new()?.subscribe([EventType::Window])?;
	let mut prev_closed = false;
	while running.load(Ordering::Relaxed)
		&& let Some(Ok(Event::Window(window))) = events.next()
	{
		if window.change != WindowChange::Focus
			|| !is_tiling(window.container.floating)
		{
			prev_closed = window.change == WindowChange::Close;
			continue;
		}
		// This is required to retrieve updated window geometry after a
		// close - focus otherwise packages pre-close geometry.
		let focused_node = if prev_closed {
			conn.get_tree()?.find(|x| x.focused).unwrap()
		} else {
			window.container
		};

		try_set_split(&mut conn, focused_node.id, focused_node.rect.into())?;
	}

	conn.run_command("layout default")?;
	Ok(())
}
