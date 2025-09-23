import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "@/hooks/useSettings";
import { ToggleSwitch } from "@/components/ui/ToggleSwitch";
import { SettingContainer } from "@/components/ui/SettingContainer";

type DescriptionMode = "tooltip" | "inline";

interface PIIRedactionProps {
  descriptionMode?: DescriptionMode;
  grouped?: boolean;
}

const PII_ENTITY_OPTIONS = [
  { value: "person", label: "Personal Names", description: "Names of people (e.g., John Smith)" },
  { value: "phone", label: "Phone Numbers", description: "Phone numbers in any format" },
  { value: "address", label: "Physical Addresses", description: "Street addresses and locations" },
  { value: "social_security_number", label: "Social Security Numbers", description: "SSN and similar government IDs" },
];

export const PIIRedaction: React.FC<PIIRedactionProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { settings, isLoading, updateSetting } = useSettings();
  const [modelLoading, setModelLoading] = useState(false);
  const [isModelLoaded, setIsModelLoaded] = useState(false);

  useEffect(() => {
    checkModelStatus();
  }, []);

  useEffect(() => {
    // Load model automatically if PII redaction is enabled at startup
    if (settings?.pii_redaction_enabled && !isModelLoaded && !modelLoading) {
      setModelLoading(true);
      invoke("load_pii_model")
        .then(() => {
          setIsModelLoaded(true);
        })
        .catch((error) => {
          console.error("Failed to auto-load PII model:", error);
        })
        .finally(() => {
          setModelLoading(false);
        });
    }
  }, [settings?.pii_redaction_enabled, isModelLoaded, modelLoading]);

  const checkModelStatus = async () => {
    try {
      const loaded = await invoke<boolean>("is_pii_model_loaded");
      setIsModelLoaded(loaded);
    } catch (error) {
      console.error("Failed to check PII model status:", error);
    }
  };


  const handleToggleRedaction = async (enabled: boolean) => {
    try {
      await updateSetting("pii_redaction_enabled", enabled);

      // Load model if enabling redaction and model not loaded
      if (enabled && !isModelLoaded && !modelLoading) {
        setModelLoading(true);
        try {
          await invoke("load_pii_model");
          setIsModelLoaded(true);
        } catch (error) {
          console.error("Failed to load PII model:", error);
        } finally {
          setModelLoading(false);
        }
      }
    } catch (error) {
      console.error("Failed to toggle PII redaction:", error);
    }
  };

  const handleToggleEntityLabels = async (enabled: boolean) => {
    try {
      await updateSetting("show_pii_entity_labels", enabled);
    } catch (error) {
      console.error("Failed to toggle entity labels:", error);
    }
  };

  const handleEntityToggle = async (entityValue: string, enabled: boolean) => {
    const currentEntities = settings?.pii_entities || [];
    let newEntities: string[];

    if (enabled) {
      newEntities = [...currentEntities, entityValue];
    } else {
      newEntities = currentEntities.filter(e => e !== entityValue);
    }

    try {
      await updateSetting("pii_entities", newEntities);
    } catch (error) {
      console.error("Failed to update PII entities:", error);
    }
  };


  if (isLoading || !settings) {
    return (
      <div className="flex justify-center p-4">
        <div className="w-6 h-6 border-2 border-logo-primary border-t-transparent rounded-full animate-spin"></div>
      </div>
    );
  }

  const containerClass = grouped
    ? "flex items-center justify-between px-4 py-3"
    : "flex items-center justify-between px-4 py-3 rounded-lg border border-mid-gray/20";

  return (
    <div className="space-y-0">
      {/* Main PII Redaction Toggle */}
      <div className={containerClass}>
        <div className="flex items-center gap-2">
          <h3 className="text-sm font-medium">PII Redaction</h3>
          {descriptionMode === "tooltip" && (
            <svg
              className="w-4 h-4 text-mid-gray cursor-help"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
              title="Automatically detect and redact personal information before output"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              />
            </svg>
          )}
        </div>
        <label className="inline-flex items-center cursor-pointer">
          <input
            type="checkbox"
            value=""
            className="sr-only peer"
            checked={settings.pii_redaction_enabled || false}
            disabled={modelLoading}
            onChange={(e) => handleToggleRedaction(e.target.checked)}
          />
          <div className="relative w-11 h-6 bg-mid-gray/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-logo-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-background-ui peer-disabled:opacity-50"></div>
          {modelLoading && (
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="w-4 h-4 border-2 border-logo-primary border-t-transparent rounded-full animate-spin"></div>
            </div>
          )}
        </label>
      </div>

      {settings.pii_redaction_enabled && (
        <div className={`space-y-4 ${grouped ? "px-4 py-2" : "p-4 border border-mid-gray/20 rounded-lg mt-4"}`}>
          <div className="space-y-4">
            {/* Show Entity Labels Toggle */}
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <h4 className="text-sm font-medium text-gray-900">Show Entity Labels</h4>
                <svg
                  className="w-4 h-4 text-mid-gray cursor-help"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                  title="Show generic labels like [PERSON] instead of hashtags when redacting"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                  />
                </svg>
              </div>
              <label className="inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  value=""
                  className="sr-only peer"
                  checked={settings.show_pii_entity_labels || false}
                  onChange={(e) => handleToggleEntityLabels(e.target.checked)}
                />
                <div className="relative w-11 h-6 bg-mid-gray/20 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-logo-primary rounded-full peer peer-checked:after:translate-x-full rtl:peer-checked:after:-translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:start-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-background-ui"></div>
              </label>
            </div>

            {/* Entity Type Selection */}
            <div className="space-y-3">
              <h4 className="text-sm font-medium text-gray-700">
                Types of information to redact:
              </h4>
              <div className="grid grid-cols-2 gap-3">
                {PII_ENTITY_OPTIONS.map((option) => (
                  <label
                    key={option.value}
                    className="flex items-start space-x-3 text-sm cursor-pointer hover:bg-gray-50 p-3 rounded border border-gray-200"
                  >
                    <input
                      type="checkbox"
                      checked={settings.pii_entities?.includes(option.value) || false}
                      onChange={(e) => handleEntityToggle(option.value, e.target.checked)}
                      className="mt-1 h-5 w-5 text-logo-primary focus:ring-logo-primary focus:ring-2 border-gray-300 rounded"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="font-medium text-gray-900">{option.label}</div>
                      <div className="text-gray-500 text-xs mt-1">{option.description}</div>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};