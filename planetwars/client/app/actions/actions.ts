import { Match, PlayingMatch, MatchId, IMapMeta, PlayerMap } from '../utils/GameModels';
import { IBotConfig, IBotData, isBotConfig, BotID, IMatchConfig } from '../utils/ConfigModels';
import { Notification } from '../utils/UtilModels';
import GameRunner from '../utils/GameRunner';
import { Config } from '../utils/Config';
import { v4 as uuidv4 } from 'uuid';
import { push } from 'react-router-redux';

import { actionCreator, actionCreatorVoid } from './helpers';
import { IGState } from '../reducers';
import { parseLog } from '../lib/match/log';
// Nav
export const toggleNavMenu = actionCreatorVoid('TOGGLE_NAV_MENU');

// About

// Bots
export type UUID = string;
export const importBotFromDB = actionCreator<IBotData>('IMPORT_BOT_FROM_DB');
export const addBot = actionCreator<IBotConfig>('ADD_BOT');
export const editBot = actionCreator<IBotData>('EDIT_BOT');
export const removeBot = actionCreator<UUID>('REMOVE_BOT');

// Matches
export const importMatchFromDB = actionCreator<Match>('IMPORT_MATCH_FROM_DB');
export const importMatchError = actionCreator<string>('IMPORT_MATCH_ERROR');
export const importMatch = actionCreator<Match>('IMPORT_MATCH');

export interface MatchParams {
  bots: BotID[];
  map: string;
  max_turns: number;
}

export const saveMatch = actionCreator<Match>('SAVE_MATCH');
export const matchErrored = actionCreator<MatchId>('MATCH_ERROR');

export function runMatch(params: MatchParams) {
  // TODO: properly type this
  return (dispatch: any, getState: any) => {
    // TODO: split this logic
    let matchId = uuidv4();

    let match: Match = {
      status: 'playing',
      uuid: matchId,
      players: params.bots,
      map: params.map,
      timestamp: new Date(),
      logPath: Config.matchLogPath(matchId),
    };

    const state: IGState = getState();

    const config: IMatchConfig = {
      players: params.bots.map((botID) => {
        return state.bots[botID].config;
      }),
      game_config: {
        map_file: state.maps[params.map].mapPath,
        max_turns: params.max_turns,
      },
      log_file: match.logPath,
    };

    dispatch(saveMatch(match));
    // TODO: ideally we'd have a separate action for this
    dispatch(push(`/matches/${matchId}`));

    let runner = new GameRunner(config);

    runner.on('matchEnded', () => {
      dispatch(completeMatch(matchId));
      const title = 'Match ended';
      const body = `A match on map '${state.maps[params.map].name}' has ended`;
      const link = `/matches/${matchId}`;
      dispatch(addNotification({ title, body, link, type: 'Finished' }));
    });
    runner.on('error', (error) => {
      dispatch(handleMatchError(matchId, error));
      const title = 'Match errored';
      const body = `A match on map '${state.maps[params.map].name}' has errored`;
      const link = `/matches/${matchId}`;
      dispatch(addNotification({ title, body, link, type: 'Error' }));
    });
    runner.run();
  }
}

export function completeMatch(matchId: MatchId) {
  return (dispatch: any, getState: any) => {
    const state: IGState = getState();
    const match = state.matches[matchId];
    if (match.status === 'playing') {
      const matchPlayers = match.players.map((uuid) => {
        const botData = state.bots[uuid];
        return {
          uuid,
          name: botData.config.name,
        };
      });
      const log = parseLog(matchPlayers, match.logPath);
      // TODO: this should go somewhere else
      // calc stats
      const winners = Array.from(log.getWinners()).map((player) => {
        return player.uuid;
      });
      const score: PlayerMap<number> = {};
      log.players.forEach((player) => {
        score[player.uuid] = player.score;
      });

      dispatch(saveMatch({
        ...match,
        status: 'finished',
        stats: {
          winners,
          score,
        },
      }));
    }
  };
}

export function handleMatchError(matchId: MatchId, error: Error) {
  return (dispatch: any, getState: any) => {
    const state: IGState = getState();
    const match = state.matches[matchId];
    if (match.status === 'playing') {
      dispatch(saveMatch({
        ...match,
        status: 'error',
        // TODO: include more information or something
        error: error.message,
      }));
    }
  };
}

// Map
export const importMapFromDB = actionCreator<IMapMeta>('IMPORT_MAP_FROM_DB');
export const importMap = actionCreator<IMapMeta>('IMPORT_MAP');
export const importMapError = actionCreator<string>('IMPORT_MAP_ERROR');

// PlayPage / Setting up a match
export const selectBot = actionCreator<BotID>('SELECT_BOT');
export const unselectBot = actionCreator<BotID>('UNSELECT_BOT');
export const unselectBotAll = actionCreator<BotID>('UNSELECT_BOT_ALL');

// DB
export const dbError = actionCreator<any>('DB_ERROR');
export const dbSync = actionCreator<any>('DB_SYNC');

// Notifications
export const importNotificationFromDB = actionCreator<Notification>('IMPORT_NOTIFICATION_FROM_DB');
export const addNotification = actionCreator<Notification>('ADD_NOTIFICATION');
export const removeNotification = actionCreator<number>('REMOVE_NOTIFICATION');
export const clearNotifications = actionCreatorVoid('CLEAR_NOTIFICATION');
export const showNotifications = actionCreatorVoid('NOTIFICATION_SHOW');
export const hideNotifications = actionCreatorVoid('NOTIFICATION_HIDE');
export const toggleNotifications = actionCreatorVoid('NOTIFICATION_TOGGLE');
