import React from "react";
import { ToggleSwitch } from "../ui/ToggleSwitch";
import { useSettings } from "../../hooks/useSettings";

interface ApiServerProps {
  descriptionMode?: "inline" | "tooltip";
  grouped?: boolean;
}

export const ApiServer: React.FC<ApiServerProps> = React.memo(({
  descriptionMode = "tooltip",
  grouped = false,
}) => {
  const { getSetting, updateSetting, isUpdating } = useSettings();

  const enableApiServer = getSetting("enable_api_server") ?? false;

  return (
    <div className="space-y-2">
      <ToggleSwitch
        checked={enableApiServer}
        onChange={(enabled) => updateSetting("enable_api_server", enabled)}
        isUpdating={isUpdating("enable_api_server")}
        label="Enable API Server"
        description="Start local server to receive transcription results via WebSocket."
        descriptionMode={descriptionMode}
        grouped={grouped}
      />
      {enableApiServer && (
        <div className="ml-4 pl-4 border-l border-gray-300 dark:border-gray-600">
          <p className="text-sm text-gray-600 dark:text-gray-400">
            API running at: <code className="bg-gray-100 dark:bg-gray-800 px-1 py-0.5 rounded text-xs">ws://localhost:7878/ws</code>
          </p>
        </div>
      )}
    </div>
  );
});