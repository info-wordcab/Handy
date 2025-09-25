import React, { useEffect, useRef, useState } from "react";

interface InfoTooltipProps {
  description: string;
  className?: string;
}

export const InfoTooltip: React.FC<InfoTooltipProps> = ({
  description,
  className = "",
}) => {
  const [showTooltip, setShowTooltip] = useState(false);
  const tooltipRef = useRef<HTMLDivElement>(null);

  // Handle click outside to close tooltip
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (
        tooltipRef.current &&
        !tooltipRef.current.contains(event.target as Node)
      ) {
        setShowTooltip(false);
      }
    };

    if (showTooltip) {
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }
  }, [showTooltip]);

  const toggleTooltip = () => {
    setShowTooltip(!showTooltip);
  };

  return (
    <div
      ref={tooltipRef}
      className={`relative ${className}`}
      onMouseEnter={() => setShowTooltip(true)}
      onMouseLeave={() => setShowTooltip(false)}
      onClick={toggleTooltip}
    >
      <svg
        className="w-4 h-4 text-mid-gray cursor-help hover:text-logo-primary transition-colors duration-200 select-none"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
        aria-label="More information"
        role="button"
        tabIndex={0}
        onKeyDown={(e) => {
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            toggleTooltip();
          }
        }}
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
        />
      </svg>
      {showTooltip && (
        <div className="absolute bottom-full left-1/2 transform -translate-x-1/2 mb-2 px-3 py-2 bg-background border border-mid-gray/80 rounded-lg shadow-lg z-50 max-w-xs min-w-[200px] whitespace-normal animate-in fade-in-0 zoom-in-95 duration-200">
          <p className="text-sm text-center leading-relaxed">
            {description}
          </p>
          <div className="absolute top-full left-1/2 transform -translate-x-1/2 w-0 h-0 border-l-[6px] border-r-[6px] border-t-[6px] border-l-transparent border-r-transparent border-t-mid-gray/80"></div>
        </div>
      )}
    </div>
  );
};