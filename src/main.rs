use std::{
	fmt::{self, Display},
	net::Shutdown,
	os::unix::net::UnixStream,
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
	ctrlc::set_handler(move || r.store(false, Ordering::Relaxed))?;

	let stream = UnixStream::from(Connection::new()?);
	let event_stream = stream.try_clone()?;
	let mut conn = Connection::from(stream);
	let mut prev_closed = false;

	let workspace_default_splits = conn
		.get_workspaces()?
		.into_iter()
		.map(|w| (w.num, w.layout))
		.collect::<Vec<_>>();

	for window in Connection::from(event_stream)
		.subscribe([EventType::Window])?
		// Here, e.is_ok() prevents Err instances from being skipped, such as
		// when Sway closes. See JayceFayne/swayipc-rs#48.
		.take_while(|e| running.load(Ordering::Relaxed) && e.is_ok())
		.flatten()
		.map(|e| if let Event::Window(w) = e { w } else { unreachable!() })
		.filter(|w| w.container.visible == Some(true))
	{
		if !matches!(window.change, WindowChange::Focus | WindowChange::Move)
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

	for (workspace, layout) in workspace_default_splits {
		conn.run_command(format!("[workspace=\"{workspace}\"] {layout}"))?;
	}

	UnixStream::from(conn).shutdown(Shutdown::Both)?;
	Ok(())
}
