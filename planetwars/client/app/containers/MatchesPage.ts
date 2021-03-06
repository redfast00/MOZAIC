import * as fs from 'mz/fs';
import * as p from 'path';
import * as Promise from 'bluebird';
import { connect, Dispatch } from 'react-redux';
import { push } from 'react-router-redux';

import Matches, {
  MatchViewerProps,
} from '../components/matches/Matches';
import { Match } from '../components/matches/types';
import { IGState } from '../reducers/index';
import { Config } from '../utils/Config';
import * as A from '../actions/actions';
import { PathLike } from 'mz/fs';

interface StateProps {
  selectedMatch?: Match;
  matches: Match[];
}

function mapStateToProps(state: IGState, ownProps: any): StateProps {
  const matches = Object.keys(state.matches).map((matchId) => {
    return getMatchData(state, matchId);
  });
  // sort descending on time
  matches.sort((a, b) => {
    return b.timestamp.getTime() - a.timestamp.getTime();
  });

  const selectedId: string | undefined = ownProps.match.params.matchId;
  if (selectedId && state.matches[selectedId]) {
    return {
      matches,
      selectedMatch: getMatchData(state, selectedId),
    };
  } else {
    return { matches };
  }
}

interface DispatchProps {
  selectMatch: (matchId: string) => void;
}

function mapDispatchToProps(dispatch: any): DispatchProps {
  return {
    selectMatch: (matchId: string) => {
      dispatch(push(`/matches/${matchId}`));
    },
  };
}

const getMatchData = (state: IGState, matchId: string) => {
  const matchData = state.matches[matchId];
  const mapData = state.maps[matchData.map];

  return {...matchData,
    players: matchData.players.map((botId) => getBotData(state, botId)),
    map: {
      uuid: mapData.uuid,
      name: mapData.name,
    },
  };
};

const getBotData = (state: IGState, botId: string) => {
  const bot = state.bots[botId];
  return {
    uuid: botId,
    name: bot.config.name,
  };
};

export default connect(mapStateToProps, mapDispatchToProps)(Matches);
