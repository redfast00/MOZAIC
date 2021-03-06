import * as React from 'react';
import { RouteComponentProps } from 'react-router';
import * as Promise from 'bluebird';
import { h } from 'react-hyperscript-helpers';
import { PlayPage, IPlayPageStateProps, IPlayPageDispatchProps } from '../components/play2/PlayPage';
import { connect } from 'react-redux';
import { fs } from 'mz';
import { v4 as uuidv4 } from 'uuid';

import { Importer } from '../utils/Importer';
import * as A from '../actions/actions';
import { IGState } from '../reducers';
import { BotID, IMatchConfig } from '../utils/ConfigModels';
import { unselectBot } from '../actions/actions';

const mapStateToProps = (state: IGState) => {
  const bots = state.bots;
  const selectedBots = state.playPage.selectedBots;
  const maps = state.maps;
  return { bots, selectedBots, maps };
};

const mapDispatchToProps = (dispatch: any) => {
  return {
    selectBot(uuid: BotID) {
      dispatch(A.selectBot(uuid));
    },
    unselectBot(uuid: BotID, all: boolean = false) {
      if (all) {
        dispatch(A.unselectBotAll(uuid));
      } else {
        dispatch(A.unselectBot(uuid));
      }
    },
    importMatch(fileList: FileList) {
      const files = Array.from(fileList); // Fuck FileList;
      const imports = files.map((logFile) => {
        const path = (logFile as any).path;
        return Importer
          .importMapFromFile(path)
          .then((map) => dispatch(A.importMapFromDB(map)))
          .catch((err) => dispatch(A.importMapError(err)));
      });
      Promise.all(imports); // TODO: Check error handling
    },
    runMatch(params: A.MatchParams) {
      dispatch(A.runMatch(params));
    },
  };
};

export default connect<IPlayPageStateProps, IPlayPageDispatchProps>(mapStateToProps, mapDispatchToProps)(PlayPage);
