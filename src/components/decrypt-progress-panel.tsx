import type { DecryptProgress, DecryptResult } from "../types";
import ProgressPanel from "./progress-panel";

interface Props {
  progress: DecryptProgress[];
  result: DecryptResult | null;
  isRunning: boolean;
}

export default function DecryptProgressPanel({ progress, result, isRunning }: Props) {
  return (
    <ProgressPanel
      progress={progress}
      result={result}
      isRunning={isRunning}
      processingVerb="decrypting…"
    />
  );
}
