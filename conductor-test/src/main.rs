use conductor::{
	id::{MetronomeId, SoundId},
	manager::{AudioManager, AudioManagerSettings, InstanceSettings},
	project::Project,
};
use ggez::{
	event::{KeyCode, KeyMods},
	graphics, Context, GameResult,
};
use std::error::Error;

struct MainState {
	audio_manager: AudioManager,
	sound_id: SoundId,
	metronome_id: MetronomeId,
}

impl MainState {
	pub fn new() -> Result<Self, Box<dyn Error>> {
		let mut project = Project::new();
		let sound_id =
			project.load_sound(&std::env::current_dir().unwrap().join("assets/hhclosed.ogg"))?;
		let metronome_id = project.create_metronome(128.0);
		Ok(Self {
			audio_manager: AudioManager::new(project, AudioManagerSettings::default())?,
			sound_id,
			metronome_id,
		})
	}
}

impl ggez::event::EventHandler for MainState {
	fn update(&mut self, _ctx: &mut Context) -> GameResult {
		Ok(())
	}

	fn key_down_event(
		&mut self,
		_ctx: &mut Context,
		keycode: KeyCode,
		_keymods: KeyMods,
		_repeat: bool,
	) {
		match keycode {
			KeyCode::Space => {
				self.audio_manager
					.start_metronome(self.metronome_id)
					.unwrap();
			}
			_ => {}
		}
	}

	fn draw(&mut self, ctx: &mut Context) -> GameResult {
		graphics::clear(ctx, graphics::BLACK);
		graphics::present(ctx)?;
		Ok(())
	}
}

fn main() -> Result<(), Box<dyn Error>> {
	let (mut ctx, mut event_loop) = ggez::ContextBuilder::new("conductor-test", "tesselode")
		.modules(ggez::conf::ModuleConf {
			audio: false,
			..Default::default()
		})
		.build()?;
	let mut main_state = MainState::new()?;
	ggez::event::run(&mut ctx, &mut event_loop, &mut main_state)?;
	Ok(())
}
