import React, { useState } from 'react';
import { ItemIcon } from './ItemIcon';
import { X, Search, Upload } from 'lucide-react';

const AVAILABLE_ICONS = [
  'Rocket',
  'BookOpen',
  'Shield',
  'FlaskConical',
  'Trophy',
  'Plane',
  'Globe',
  'Landmark',
  'Smartphone',
  'Compass',
  'Orbit',
  'Film',
  'Building2',
  'Clock',
  'Radio',
  'Triangle',
  'CableCar',
  'HelpCircle',
];

interface IconPickerModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectIcon: (iconName: string, customUrl?: string) => void;
  currentIcon?: string;
}

export const IconPickerModal: React.FC<IconPickerModalProps> = ({
  isOpen,
  onClose,
  onSelectIcon,
  currentIcon,
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [customUrl, setCustomUrl] = useState('');

  if (!isOpen) return null;

  const filteredIcons = AVAILABLE_ICONS.filter((name) =>
    name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (uploadEvent) => {
        const result = uploadEvent.target?.result as string;
        if (result) {
          onSelectIcon('custom', result);
          onClose();
        }
      };
      reader.readAsDataURL(file);
    }
  };

  return (
    <div className="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-4">
      <div className="bg-slate-900 border border-slate-700/80 rounded-2xl p-5 max-w-md w-full shadow-2xl">
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-bold text-white uppercase tracking-wider font-['Syne']">
            Select or Upload Icon
          </h3>
          <button
            onClick={onClose}
            className="text-slate-400 hover:text-white transition-colors p-1"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Search */}
        <div className="relative mb-4">
          <Search className="w-3.5 h-3.5 text-slate-500 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            placeholder="Search icons..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="w-full bg-slate-950 border border-slate-700 rounded-lg pl-8 pr-3 py-1.5 text-xs text-slate-200 placeholder-slate-500 focus:outline-none focus:border-cyan-400"
          />
        </div>

        {/* Grid of standard icons */}
        <div className="grid grid-cols-6 gap-2 max-h-48 overflow-y-auto p-1 mb-4">
          {filteredIcons.map((name) => {
            const isSelected = currentIcon === name;
            return (
              <button
                key={name}
                type="button"
                onClick={() => {
                  onSelectIcon(name);
                  onClose();
                }}
                className={`p-2.5 rounded-lg flex flex-col items-center justify-center gap-1 transition-all ${
                  isSelected
                    ? 'bg-cyan-500 text-slate-950 ring-2 ring-cyan-300'
                    : 'bg-slate-800 text-slate-300 hover:bg-slate-700 hover:text-white'
                }`}
                title={name}
              >
                <ItemIcon name={name} className="w-5 h-5" />
              </button>
            );
          })}
        </div>

        {/* Custom Upload or URL */}
        <div className="pt-3 border-t border-slate-800 text-xs">
          <label className="block text-slate-400 font-semibold mb-2">Or Upload Image / SVG</label>
          <div className="flex items-center gap-2">
            <label className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 cursor-pointer text-xs font-semibold">
              <Upload className="w-3.5 h-3.5 text-cyan-400" />
              <span>Upload File</span>
              <input
                type="file"
                accept="image/*,.svg"
                onChange={handleFileUpload}
                className="hidden"
              />
            </label>
            <span className="text-[11px] text-slate-500">PNG, SVG, or JPG</span>
          </div>
        </div>
      </div>
    </div>
  );
};
