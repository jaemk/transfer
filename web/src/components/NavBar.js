import React, { Component } from 'react';
import PropTypes from 'prop-types';
import Nav from 'react-bootstrap/lib/Nav';


const URL_TO_INDEX = {
  '/upload': '1',
  '/download': '2',
  '/delete': '3',
};

const INDEX_TO_URL = (() => {
  let m = {};
  Object.keys(URL_TO_INDEX).map(url => m[URL_TO_INDEX[url]] = url);
  return m;
})();


class NavBar extends Component {
  static propTypes = {
    history: PropTypes.object.isRequired,
  }

  constructor(props) {
    super(props);
    this.mapUrlToNavKey = this.mapUrlToNavKey.bind(this);
    this.handleSelect = this.handleSelect.bind(this);
  }

  mapUrlToNavKey(url) {
    return URL_TO_INDEX[url];
  }

  handleSelect(eventKey) {
    const url = INDEX_TO_URL[eventKey] || '/upload';
    this.props.history.push(url);
  }

  render() {
    const activeKey = this.mapUrlToNavKey(this.props.history.location.pathname);
    const isDisabled = (eventKey) => activeKey === eventKey;
    return (
      <div>
        <Nav variant="tabs" activeKey={activeKey} onSelect={this.handleSelect}>
          <Nav.Item>
            <Nav.Link eventKey="1" disabled={isDisabled('1')}>
              Upload
            </Nav.Link>
          </Nav.Item>
          <Nav.Item>
            <Nav.Link eventKey="2" disabled={isDisabled('2')}>
              Download
            </Nav.Link>
          </Nav.Item>
          <Nav.Item>
            <Nav.Link eventKey="3" disabled={isDisabled('3')}>
              Delete
            </Nav.Link>
          </Nav.Item>
        </Nav>
      </div>
    )
  }
}

export default NavBar;

