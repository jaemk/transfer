import React from 'react';
import ReactDOM from 'react-dom';
import injectTapEventPlugin from 'react-tap-event-plugin';
import { Router, Route, Switch, Redirect } from 'react-router-dom';
import { createHashHistory } from 'history';
import './index.css';
import NavBar from './components/NavBar';
import Upload from './components/Upload';
import Download from './components/Download';
import Delete from './components/Delete';
import NotFound from './components/NotFound';
import 'bootstrap/dist/css/bootstrap.css';
injectTapEventPlugin();

const history = createHashHistory({
  basename: '',
  hashType: 'slash',
});

const style = {
  padding: '10px 30px',
};


const uploadRedirect = () => {
  return (
    <Redirect to="/upload"/>
  );
};


ReactDOM.render(
  <Router history={history}>
    <div style={style}>
      <h2> Transfer </h2>

      <NavBar
        history={history}
      />

      <Switch>
        <Route exact path="/" component={uploadRedirect}/>
        <Route path="/upload" component={Upload}/>
        <Route path="/download" component={Download}/>
        <Route path="/delete" component={Delete}/>
        <Route path="*" component={NotFound}/>
      </Switch>
    </div>
  </Router>,
  document.getElementById('root'));

