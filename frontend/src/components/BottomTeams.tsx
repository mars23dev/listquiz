import React from 'react';
import { motion } from 'motion/react';
import { Team, UserRole } from '../types/quiz';
import { Heart, User } from 'lucide-react';

interface BottomTeamsProps {
  teams: Team[];
  activeTeamCount: number;
  currentRole?: UserRole;
  /** Hearts are an Order mechanic; a Tripmine round hides them. */
  showHearts?: boolean;
}

export const BottomTeams: React.FC<BottomTeamsProps> = ({
  teams,
  activeTeamCount,
  currentRole,
  showHearts = true,
}) => {
  const displayedTeams = teams.slice(0, activeTeamCount);

  // Determine which team index corresponds to current player role
  const playerTeamIndex = currentRole?.startsWith('p') ? parseInt(currentRole.replace('p', '')) - 1 : -1;

  const getTeamCardStyle = (colorName: Team['colorName'], isActive: boolean, isEliminated: boolean, isCurrentPlayer: boolean) => {
    if (isEliminated) {
      return {
        card: 'border-slate-800/80 bg-slate-950/40 opacity-35 grayscale cursor-not-allowed shadow-none',
        scoreText: 'text-slate-500',
        nameText: 'text-slate-500',
      };
    }

    switch (colorName) {
      case 'cyan':
        return {
          card: isActive
            ? 'border-cyan-400 ring-2 ring-cyan-400/60 shadow-xl shadow-cyan-500/30 bg-cyan-950/40'
            : isCurrentPlayer
            ? 'border-cyan-500/60 ring-1 ring-cyan-500/30 bg-slate-900/95'
            : 'border-cyan-500/30 hover:border-cyan-400/50 bg-slate-900/90',
          scoreText: 'text-cyan-300',
          nameText: 'text-slate-100',
        };
      case 'rose':
        return {
          card: isActive
            ? 'border-rose-400 ring-2 ring-rose-400/60 shadow-xl shadow-rose-500/30 bg-rose-950/40'
            : isCurrentPlayer
            ? 'border-rose-500/60 ring-1 ring-rose-500/30 bg-slate-900/95'
            : 'border-rose-500/30 hover:border-rose-400/50 bg-slate-900/90',
          scoreText: 'text-rose-300',
          nameText: 'text-slate-100',
        };
      case 'amber':
        return {
          card: isActive
            ? 'border-amber-400 ring-2 ring-amber-400/60 shadow-xl shadow-amber-500/30 bg-amber-950/40'
            : isCurrentPlayer
            ? 'border-amber-500/60 ring-1 ring-amber-500/30 bg-slate-900/95'
            : 'border-amber-500/30 hover:border-amber-400/50 bg-slate-900/90',
          scoreText: 'text-amber-300',
          nameText: 'text-slate-100',
        };
      case 'emerald':
      default:
        return {
          card: isActive
            ? 'border-emerald-400 ring-2 ring-emerald-400/60 shadow-xl shadow-emerald-500/30 bg-emerald-950/40'
            : isCurrentPlayer
            ? 'border-emerald-500/60 ring-1 ring-emerald-500/30 bg-slate-900/95'
            : 'border-emerald-500/30 hover:border-emerald-400/50 bg-slate-900/90',
          scoreText: 'text-emerald-300',
          nameText: 'text-slate-100',
        };
    }
  };

  return (
    <footer className="relative z-10 w-full px-2 sm:px-6 py-3 border-t border-slate-800/80 bg-slate-950/90 backdrop-blur-xl">
      <div className="max-w-7xl mx-auto">
        {/* All Teams Fit in One Single Horizontal Line */}
        <div
          className="grid gap-2.5 sm:gap-3 items-stretch"
          style={{
            gridTemplateColumns: `repeat(${activeTeamCount}, minmax(0, 1fr))`,
          }}
        >
          {displayedTeams.map((team, index) => {
            const isEliminated = showHearts && team.hearts <= 0;
            const isCurrentPlayer = playerTeamIndex === index;
            const style = getTeamCardStyle(team.colorName, team.isActiveTurn, isEliminated, isCurrentPlayer);

            return (
              <motion.div
                key={team.id}
                layout
                className={`relative flex flex-col justify-between rounded-xl border-2 p-2.5 sm:p-3 transition-all duration-200 select-none ${style.card}`}
              >
                {/* Top Row: Team Name + User symbol if current player + 3 Hearts */}
                <div className="flex items-center justify-between gap-1 mb-1.5">
                  <div className="flex items-center gap-1.5 truncate">
                    {isCurrentPlayer && (
                      <span className="flex items-center gap-0.5 px-1 py-0.2 rounded bg-cyan-500/20 text-cyan-300 border border-cyan-400/40 text-[9px] font-mono font-bold uppercase shrink-0">
                        <User className="w-2.5 h-2.5" />
                        <span>You</span>
                      </span>
                    )}
                    <h3 className={`text-xs sm:text-sm font-bold truncate font-['Syne'] ${style.nameText}`}>
                      {team.name}
                    </h3>
                  </div>

                  {/* 3 Hearts Display (Order only) */}
                  {showHearts && (
                    <div className="flex items-center gap-0.5 shrink-0" title={`${team.hearts} / 3 Hearts remaining`}>
                      {[1, 2, 3].map((heartIndex) => {
                        const hasHeart = team.hearts >= heartIndex;
                        return (
                          <Heart
                            key={heartIndex}
                            className={`w-3.5 h-3.5 transition-colors ${
                              hasHeart
                                ? 'fill-rose-500 text-rose-500 filter drop-shadow-[0_0_4px_rgba(244,63,94,0.6)]'
                                : 'fill-slate-800 text-slate-700'
                            }`}
                          />
                        );
                      })}
                    </div>
                  )}
                </div>

                {/* Score Below Team Name */}
                <div className="pt-1 border-t border-slate-800/60 flex items-center justify-between">
                  <div
                    className={`text-2xl sm:text-3xl font-black font-mono tabular-nums tracking-tight leading-none ${style.scoreText}`}
                  >
                    {team.score.toLocaleString()}
                  </div>

                  {isEliminated ? (
                    <span className="text-[10px] font-mono uppercase tracking-wider text-rose-500/80 font-bold">
                      Out
                    </span>
                  ) : team.isActiveTurn ? (
                    <span className="text-[10px] font-mono uppercase tracking-wider text-cyan-400 font-bold animate-pulse">
                      Turn
                    </span>
                  ) : null}
                </div>
              </motion.div>
            );
          })}
        </div>
      </div>
    </footer>
  );
};

