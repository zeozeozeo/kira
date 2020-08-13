use crate::{
	command::InstanceCommand,
	instance::{Instance, InstanceId},
	sound::{Sound, SoundId},
	stereo_sample::StereoSample,
	tween::Tween,
};
use indexmap::IndexMap;

pub(crate) struct Instances {
	instances: IndexMap<InstanceId, Instance>,
	instances_to_remove: Vec<InstanceId>,
}

impl Instances {
	pub fn new(capacity: usize) -> Self {
		Self {
			instances: IndexMap::with_capacity(capacity),
			instances_to_remove: Vec::with_capacity(capacity),
		}
	}

	pub fn stop_instances_of_sound(&mut self, id: SoundId, fade_tween: Option<Tween>) {
		for (_, instance) in &mut self.instances {
			if instance.sound_id == id {
				instance.stop(fade_tween);
			}
		}
	}

	pub fn run_command(&mut self, command: InstanceCommand) {
		match command {
			InstanceCommand::PlaySound(sound_id, instance_id, settings) => {
				self.instances
					.insert(instance_id, Instance::new(sound_id, settings));
			}
			InstanceCommand::SetInstanceVolume(id, volume, tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.set_volume(volume, tween);
				}
			}
			InstanceCommand::SetInstancePitch(id, pitch, tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.set_pitch(pitch, tween);
				}
			}
			InstanceCommand::PauseInstance(id, fade_tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.pause(fade_tween);
				}
			}
			InstanceCommand::ResumeInstance(id, fade_tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.resume(fade_tween);
				}
			}
			InstanceCommand::StopInstance(id, fade_tween) => {
				if let Some(instance) = self.instances.get_mut(&id) {
					instance.stop(fade_tween);
				}
			}
			InstanceCommand::PauseInstancesOfSound(id, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.sound_id == id {
						instance.pause(fade_tween);
					}
				}
			}
			InstanceCommand::ResumeInstancesOfSound(id, fade_tween) => {
				for (_, instance) in &mut self.instances {
					if instance.sound_id == id {
						instance.resume(fade_tween);
					}
				}
			}
			InstanceCommand::StopInstancesOfSound(id, fade_tween) => {
				self.stop_instances_of_sound(id, fade_tween);
			}
		}
	}

	pub fn process(&mut self, dt: f32, sounds: &IndexMap<SoundId, Sound>) -> StereoSample {
		let mut out = StereoSample::from_mono(0.0);
		for (instance_id, instance) in &mut self.instances {
			if instance.playing() {
				if let Some(sound) = sounds.get(&instance.sound_id) {
					out += sound.get_sample_at_position(instance.position())
						* instance.effective_volume();
				}
			}
			if instance.finished() {
				self.instances_to_remove.push(*instance_id);
			}
			instance.update(dt);
		}
		for instance_id in self.instances_to_remove.drain(..) {
			self.instances.remove(&instance_id);
		}
		out
	}
}