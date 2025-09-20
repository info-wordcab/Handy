import React, { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useSettings } from "@/hooks/useSettings";
import { cn } from "@/lib/utils";
import { Checkbox } from "@/components/ui/checkbox";
import { Switch } from "@/components/ui/switch";
import { Loader2 } from "lucide-react";

type DescriptionMode = "tooltip" | "inline";

interface PIIRedactionProps {
  descriptionMode?: DescriptionMode;
  grouped?: boolean;
}

const PII_ENTITY_OPTIONS = [
  { value: "person", label: "Names", description: "Personal names" },
  { value: "email", label: "Email Addresses", description: "Email addresses" },
  { value: "phone_number", label: "Phone Numbers", description: "Phone numbers" },
  { value: "social_security_number", label: "SSN", description: "Social Security Numbers" },
  { value: "credit_card", label: "Credit Cards", description: "Credit card numbers" },
  { value: "address", label: "Addresses", description: "Physical addresses" },
  { value: "date_of_birth", label: "Birth Dates", description: "Dates of birth" },
  { value: "passport_number", label: "Passport", description: "Passport numbers" },
  { value: "driver_license", label: "Driver's License", description: "Driver's license numbers" },
  { value: "bank_account", label: "Bank Account", description: "Bank account numbers" },
];

export const PIIRedaction: React.FC<PIIRedactionProps> = ({
  descriptionMode = "inline",
  grouped = false,
}) => {
  const { settings, loading, setSettings } = useSettings();
  const [modelLoading, setModelLoading] = useState(false);
  const [isModelLoaded, setIsModelLoaded] = useState(false);

  useEffect(() => {
    checkModelStatus();
  }, []);

  const checkModelStatus = async () => {
    try {
      const loaded = await invoke<boolean>("is_pii_model_loaded");
      setIsModelLoaded(loaded);
    } catch (error) {
      console.error("Failed to check PII model status:", error);
    }
  };

  if (loading || !settings) {
    return (
      <div className="flex justify-center p-4">
        <Loader2 className="h-6 w-6 animate-spin" />
      </div>
    );
  }

  const handleToggleRedaction = async (enabled: boolean) => {
    try {
      await invoke("set_pii_redaction_enabled", { enabled });
      setSettings({ ...settings, pii_redaction_enabled: enabled });

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

  const handleEntityToggle = async (entity: string, checked: boolean) => {
    const currentEntities = settings.pii_entities || [];
    const newEntities = checked
      ? [...currentEntities, entity]
      : currentEntities.filter((e) => e !== entity);

    try {
      await invoke("set_pii_entities", { entities: newEntities });
      setSettings({ ...settings, pii_entities: newEntities });
    } catch (error) {
      console.error("Failed to update PII entities:", error);
    }
  };

  const containerClass = cn(
    "space-y-4",
    grouped && "pl-6 border-l-2 border-gray-200"
  );

  return (
    <div className={containerClass}>
      <div className="flex items-center justify-between">
        <div className="flex-1">
          <label className="text-sm font-medium">PII Redaction</label>
          {descriptionMode === "inline" && (
            <p className="text-sm text-gray-500 mt-1">
              Automatically detect and redact personal information before output
            </p>
          )}
        </div>
        <div className="flex items-center gap-2">
          {modelLoading && <Loader2 className="h-4 w-4 animate-spin" />}
          <Switch
            checked={settings.pii_redaction_enabled || false}
            onCheckedChange={handleToggleRedaction}
            disabled={modelLoading}
          />
        </div>
      </div>

      {settings.pii_redaction_enabled && (
        <div className="ml-6 space-y-2">
          <p className="text-sm font-medium text-gray-700">
            Select information to redact:
          </p>
          <div className="grid grid-cols-2 gap-3">
            {PII_ENTITY_OPTIONS.map((option) => (
              <div key={option.value} className="flex items-start space-x-2">
                <Checkbox
                  id={option.value}
                  checked={settings.pii_entities?.includes(option.value) || false}
                  onCheckedChange={(checked) =>
                    handleEntityToggle(option.value, checked as boolean)
                  }
                  className="mt-0.5"
                />
                <label
                  htmlFor={option.value}
                  className="text-sm cursor-pointer select-none"
                >
                  <div className="font-medium">{option.label}</div>
                  {descriptionMode === "inline" && (
                    <div className="text-xs text-gray-500">{option.description}</div>
                  )}
                </label>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};