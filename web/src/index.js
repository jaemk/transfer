import React from 'react';
import ReactDOM from 'react-dom';
import injectTapEventPlugin from 'react-tap-event-plugin';
import { Router, Route, Switch } from 'react-router-dom';
import { createHashHistory } from 'history';
import './index.css';
import NavBar from './components/NavBar';
import Upload from './components/Upload';
import Download from './components/Download';
import Delete from './components/Delete';
import NotFound from './components/NotFound';
import registerServiceWorker from './registerServiceWorker';
import 'bootstrap/dist/css/bootstrap.css';
injectTapEventPlugin();

const history = createHashHistory({
  basename: '',
  hashType: 'slash',
});

const style = {
  padding: '10px 30px',
};

ReactDOM.render(
  <Router history={history}>
    <div style={style}>
      <h3> Transfer </h3>

      <NavBar
        history={history}
      />

      <Switch>
        <Route path="/upload" component={Upload}/>
        <Route path="/download" component={Download}/>
        <Route path="/delete" component={Delete}/>
        <Route path="*" component={NotFound}/>
      </Switch>
    </div>
  </Router>,
  document.getElementById('root'));

registerServiceWorker();

