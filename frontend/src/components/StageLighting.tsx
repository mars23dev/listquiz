import React from 'react';

export const StageLighting: React.FC = () => {
  return (
    <div className="fixed inset-0 pointer-events-none overflow-hidden select-none z-0">
      {/* Studio ceiling spotlight cones */}
      <div 
        className="absolute top-0 left-1/2 -translate-x-1/2 w-[900px] h-[500px] bg-gradient-to-b from-cyan-500/12 via-indigo-500/5 to-transparent blur-3xl opacity-70"
        aria-hidden="true" 
      />
      
      {/* Left team spotlight */}
      <div 
        className="absolute bottom-0 left-0 w-[500px] h-[450px] bg-gradient-to-tr from-cyan-600/10 via-blue-500/4 to-transparent blur-3xl opacity-60"
        aria-hidden="true" 
      />

      {/* Right team spotlight */}
      <div 
        className="absolute bottom-0 right-0 w-[500px] h-[450px] bg-gradient-to-tl from-rose-600/10 via-purple-500/4 to-transparent blur-3xl opacity-60"
        aria-hidden="true" 
      />

      {/* Studio stage floor light reflection */}
      <div 
        className="absolute bottom-0 inset-x-0 h-48 bg-gradient-to-t from-slate-900/80 via-slate-950/60 to-transparent"
        aria-hidden="true" 
      />

      {/* Subtle broadcast scanline / grid texture */}
      <div 
        className="absolute inset-0 opacity-[0.03] bg-[radial-gradient(#38bdf8_1px,transparent_1px)] [background-size:24px_24px]"
        aria-hidden="true"
      />
    </div>
  );
};
