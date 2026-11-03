import React from 'react';
import {
  Rocket,
  BookOpen,
  Shield,
  FlaskConical,
  Trophy,
  Plane,
  Globe,
  Landmark,
  Smartphone,
  Compass,
  Orbit,
  Film,
  Building2,
  Clock,
  Radio,
  Triangle,
  CableCar,
  HelpCircle,
} from 'lucide-react';

interface ItemIconProps {
  name?: string;
  customIconUrl?: string;
  className?: string;
}

export const ItemIcon: React.FC<ItemIconProps> = ({ name, customIconUrl, className = 'w-4 h-4' }) => {
  if (customIconUrl) {
    return (
      <img
        src={customIconUrl}
        alt={name || 'Item icon'}
        className={`${className} object-contain rounded`}
        referrerPolicy="no-referrer"
      />
    );
  }

  // A country flag (feature 014): `flag:<cc>` renders the bundled flag-icons SVG.
  if (name?.startsWith('flag:')) {
    const code = name.slice('flag:'.length).trim().toLowerCase();
    return <span className={`fi fi-${code} ${className}`} aria-hidden="true" />;
  }

  switch (name) {
    case 'Rocket':
      return <Rocket className={className} />;
    case 'BookOpen':
      return <BookOpen className={className} />;
    case 'Shield':
      return <Shield className={className} />;
    case 'FlaskConical':
      return <FlaskConical className={className} />;
    case 'Trophy':
      return <Trophy className={className} />;
    case 'Plane':
      return <Plane className={className} />;
    case 'Globe':
      return <Globe className={className} />;
    case 'Landmark':
      return <Landmark className={className} />;
    case 'Smartphone':
      return <Smartphone className={className} />;
    case 'Compass':
      return <Compass className={className} />;
    case 'Orbit':
      return <Orbit className={className} />;
    case 'Film':
      return <Film className={className} />;
    case 'Building2':
      return <Building2 className={className} />;
    case 'Clock':
      return <Clock className={className} />;
    case 'Radio':
      return <Radio className={className} />;
    case 'Triangle':
      return <Triangle className={className} />;
    case 'CableCar':
      return <CableCar className={className} />;
    default:
      return <HelpCircle className={className} />;
  }
};
