import { GameState, Player } from '../../../../lib/match/types';
import { MatchLog } from '../../../../lib/match/log';

import Config from '../util/config';

export default class Game {
  playerColorMap: Map<string, string>;
  planetTypeMap: Map<string, string>;
  matchLog: MatchLog;

  constructor(matchLog: MatchLog) {
    this.matchLog = matchLog;
    this.planetTypeMap = new Map();
    this.playerColorMap = new Map();
    matchLog.players.forEach((player, idx) => {
      this.playerColorMap.set(player.name, Config.playerColors[idx]);
    });
  }

  public playerColor(player?: Player): string {
    if (player) {
      return this.playerColorMap.get(player.name)!;
    }
    return Config.neutralColor;
  }

  public planetType(name: string): string {
    if (!this.planetTypeMap.has(name)) {
      var types: string[] = Config.planetTypes;
      var type: string = types[Math.floor(Math.random() * types.length)];
      this.planetTypeMap.set(name, type);
    }
    return this.planetTypeMap.get(name)!;
  }
}
