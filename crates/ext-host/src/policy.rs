use crate::wasm::PluginError;

pub const STRIKES: u32 = 3;

pub fn is_fault(e: &PluginError) -> bool {
    matches!(e, PluginError::Trap(_) | PluginError::TooSlow | PluginError::TooMuchMemory)
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Strikes(u32);

impl Strikes {
    pub fn record(&mut self, outcome: Result<(), &PluginError>) -> Option<String> {
        match outcome {
            Ok(()) => {
                self.0 = 0;
                None
            }
            Err(e) if is_fault(e) => {
                self.0 += 1;
                (self.0 >= STRIKES).then(|| format!("stopped after {STRIKES} failures in a row: {e}"))
            }
            Err(_) => None,
        }
    }

    pub fn reset(&mut self) {
        self.0 = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_faults_count_and_a_success_resets_the_count() {
        let mut s = Strikes::default();
        let trap = PluginError::Trap("unreachable".into());
        let own_error = PluginError::Failed(-1, "bad input".into());
        assert_eq!(s.record(Err(&trap)), None);
        assert_eq!(s.record(Err(&own_error)), None);
        assert_eq!(s.record(Err(&PluginError::Stopped("document changed".into()))), None);
        assert_eq!(s.record(Err(&trap)), None);
        assert_eq!(s.record(Err(&PluginError::TooSlow)), Some("stopped after 3 failures in a row: the plugin took too long".into()));
        s.reset();
        assert_eq!(s.record(Err(&trap)), None);
        assert_eq!(s.record(Ok(())), None);
        assert_eq!(s.record(Err(&trap)), None);
        assert_eq!(s.record(Err(&trap)), None);
    }
}
