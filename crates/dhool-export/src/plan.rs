use std::path::PathBuf;

use dhool_config::{ExportConfig, ExportMode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedOutputs {
    pub output_dir: PathBuf,
    pub sequence_dir: Option<PathBuf>,
    pub video_path: Option<PathBuf>,
}

pub fn plan_outputs(export: &ExportConfig, stem: &str, include_sequence: bool) -> PlannedOutputs {
    let output_dir = PathBuf::from(&export.output_dir);
    let writes_sequence =
        include_sequence && matches!(export.mode, ExportMode::Sequence | ExportMode::Both);
    let writes_video = matches!(export.mode, ExportMode::Pipe | ExportMode::Both);
    PlannedOutputs {
        output_dir: output_dir.clone(),
        sequence_dir: writes_sequence.then(|| output_dir.join("frames")),
        video_path: writes_video.then(|| output_dir.join(format!("{stem}.mp4"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dhool_config::{ExportMode, SequenceFormat};

    #[test]
    fn plans_sequence_and_video_paths() {
        let export = ExportConfig {
            mode: ExportMode::Both,
            sequence_format: SequenceFormat::Png,
            video_codec: "libx264".to_string(),
            crf: 18,
            preset: "medium".to_string(),
            output_dir: "outputs/test".to_string(),
        };
        let plan = plan_outputs(&export, "preview", true);
        assert_eq!(
            plan.sequence_dir.unwrap(),
            PathBuf::from("outputs/test/frames")
        );
        assert_eq!(
            plan.video_path.unwrap(),
            PathBuf::from("outputs/test/preview.mp4")
        );
    }

    #[test]
    fn omits_sequence_when_not_requested() {
        let export = ExportConfig {
            mode: ExportMode::Both,
            sequence_format: SequenceFormat::Png,
            video_codec: "libx264".to_string(),
            crf: 18,
            preset: "medium".to_string(),
            output_dir: "outputs/test".to_string(),
        };
        let plan = plan_outputs(&export, "preview", false);
        assert!(plan.sequence_dir.is_none());
        assert_eq!(
            plan.video_path.unwrap(),
            PathBuf::from("outputs/test/preview.mp4")
        );
    }
}
