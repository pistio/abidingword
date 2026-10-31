import { BatchImport } from "./BatchImport";

/** One review workflow for a single file, multiple files and folders. */
export function AddSourceView({ onClose }: { onClose: () => void }) {
  return <BatchImport onClose={() => onClose()} />;
}
