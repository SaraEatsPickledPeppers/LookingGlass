use super::{Batch, Cue, Receiver};
use crate::{assets::Assets, audio::Audio, bsp::Bsp, collision::World, coop::{arena::Arena, world::Frame, PlayerId}, Options};
use anyhow::Result;
use macroquad::prelude::Vec3;
use serde::Serialize;

#[derive(Default, Serialize)]
pub struct Report {
    pub accepted: u64,
    pub recent: Vec<Cue>,
    pub voice_starts: u64,
    pub voice_path: Option<String>,
    pub voice_elapsed: f32,
    pub output_frames: u64,
    pub peak: f32,
    pub resolved: u64,
    pub missing: usize,
}
#[derive(Default)]
struct Voice {
    key: Option<(String,usize,String)>,
    elapsed: f32,
}
impl Voice {
    fn changed(&mut self, now: Option<(&str,usize,&str,f32)>) -> bool {
        let key = now.map(|(id,index,path,_)|(id.to_owned(),index,path.to_owned()));
        let elapsed = now.map_or(0.,|v|v.3);
        let changed = key != self.key || elapsed + 0.25 < self.elapsed;
        self.key = key; self.elapsed = elapsed;
        changed
    }
}
pub struct Playback {
    audio: Audio,
    receiver: Receiver,
    voice: Voice,
    scope: Option<(u64,u64)>,
    capture_started: bool,
    pub report: Report,
}
impl Playback {
    pub fn new(options: &Options) -> Self {
        Self { audio:Audio::new(options.no_audio,options.audio_capture.clone()),
            receiver:Receiver::default(), voice:Voice::default(), scope:None, capture_started:false, report:Report::default() }
    }
    pub fn load(&mut self, assets:&mut Assets, name:&str, map:&Bsp) {
        self.audio.load(assets,name,map);
        self.receiver = Receiver::default(); self.voice = Voice::default(); self.scope = None;
    }
    pub fn update(&mut self, assets:&mut Assets, arena:&Arena, frame:&Frame, world:&World,
        local:PlayerId, eye:Vec3, yaw:f32, dt:f32, playing:bool) -> Result<()> {
        if playing && !self.capture_started { self.audio.begin_capture(); self.capture_started = true; }
        let scope = (frame.epoch,frame.checkpoint_generation);
        if self.scope != Some(scope) {
            self.audio.restore_world(); self.voice = Voice::default(); self.scope = Some(scope);
        }
        let empty = Batch::default();
        for cue in self.receiver.observe(scope,frame.tick,frame.sounds.as_ref().unwrap_or(&empty))? {
            if playing {
                if cue.animation {
                    self.audio.animation_at(crate::audio::events::Cue {path:cue.path.clone(),volume:cue.volume},cue.origin);
                } else { self.audio.effect(&cue.path,cue.origin,cue.volume); }
                self.report.accepted += 1;
                self.report.recent.push(cue);
                if self.report.recent.len() > 128 { self.report.recent.remove(0); }
            }
        }
        let current = playing.then(||arena.stage.story.voice_position()).flatten();
        if self.voice.changed(current) {
            if let Some((_,_,path,elapsed)) = current {
                self.audio.speak_from(assets,path,elapsed);
                self.report.voice_starts += 1;
            } else { self.audio.stop_voice(); }
        }
        self.report.voice_path = current.map(|v|v.2.to_owned());
        self.report.voice_elapsed = current.map_or(0.,|v|v.3);
        let loops = arena.characters.iter().enumerate().flat_map(|(index,c)|
            c.audio_loops().into_iter().filter_map(move |mut cue| {
                // Disjoint from level loops and the other Alice's projectile IDs.
                cue.id = cue.id.checked_mul(2)?.checked_add(index)? | (1usize << 63);
                Some(cue)
            })).collect();
        self.audio.world_replica(assets,&arena.stage.interactions,loops);
        let body = &frame.players[local.index()];
        self.audio.player_state(assets,body.stats.sanity(),body.player.swimming,
            body.player.immersion.level == 3, body.player.immersion.level == 3 && body.player.breath.remaining() <= 0.);
        self.audio.prepare(assets,world,eye,body.player.immersion.level == 3,dt);
        let world_dt = frame.players.iter().fold(dt,|dt,b|b.stats.powers.world_dt(dt));
        self.audio.update_clocks(dt,world_dt,eye,yaw,!playing,!playing || !arena.stage.voice_running);
        (self.report.output_frames,self.report.peak,self.report.resolved,self.report.missing) = self.audio.diagnostics();
        Ok(())
    }
    pub fn finish(&mut self) -> Result<()> { self.audio.finish() }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn voices_follow_lines_and_seek_only_on_join_skip_or_rewind() {
        let mut voice = Voice::default();
        assert!(!voice.changed(None));
        assert!(voice.changed(Some(("scene",0,"sound/line.wav",1.2))));
        for elapsed in [1.2,1.2,1.25,1.3] { assert!(!voice.changed(Some(("scene",0,"sound/line.wav",elapsed)))); }
        assert!(voice.changed(Some(("scene",1,"sound/line.wav",0.))));
        assert!(!voice.changed(Some(("scene",1,"sound/line.wav",3.))));
        assert!(voice.changed(Some(("scene",1,"sound/line.wav",0.5))));
        assert!(voice.changed(None)); assert!(!voice.changed(None));
    }
}
