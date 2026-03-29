import type { EncryptProgress, EncryptResult } from "../types";
import ProgressPanel from "./progress-panel";

interface Props {
  progress: EncryptProgress[];
  result: EncryptResult | null;
  isRunning: boolean;
}

export default function EncryptProgressPanel({ progress, result, isRunning }: Props) {
  return (
    <ProgressPanel
      progress={progress}
      result={result}
      isRunning={isRunning}
      processingVerb="encrypting…"
      showProcessingFileName
    />
  );
}
