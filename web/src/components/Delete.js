import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { Button, Form } from 'react-bootstrap';
import axios from 'axios';
import TextField from './TextField';
import PasswordField from './PasswordField';


class Delete extends Component {
  static propTypes = {
    location: PropTypes.object.isRequired,
  }

  constructor(props) {
    super(props);
    this.state = {
      key: '',
      deletePass: '',
      required: {},
      submitted: false,
      inputOk: false,
    };
    this.delete = this.delete.bind(this);
  }

  componentWillMount() {
    const search = this.props.location.search;
    const params = new URLSearchParams(search);
    const key = params.get('key');
    if (key) {
      this.setState({key: key});
    }
  }

  delete() {
    console.log(this.state);
    let required = {};
    if (this.state.key.length === 0) {
      required.key = true;
    }
    if (this.state.deletePass.length === 0) {
      required.deletePass = true;
    }

    let inputOk = true;
    if (Object.keys(required).length > 0) {
      inputOk = false;
    }
    this.setState({required: required, submitted: true, inputOk: inputOk});
    if (!inputOk) {
      return;
    }

    const params = {key: this.state.key, deletion_password: Buffer.from(this.state.deletePass).toString('hex')}
    const headers = {headers: {'content-type': 'application/json'}}
    axios.post('/api/upload/delete', params, headers).then(resp => {
      console.log(resp.data);
    });
  }

  render() {
    const disable = this.state.submitted && this.state.inputOk;
    return (
      <div>
        <Form inline onSubmit={this.delete}>
          <TextField
            title="Upload Key"
            value={this.state.key}
            disabled={disable}
            update={(v) => this.setState({key: v})}
            required={this.state.required.key}
          />
          <br/>

          <PasswordField
            title="Delete"
            disabled={disable}
            update={(v) => this.setState({deletePass: v})}
            required={this.state.required.deletePass}
          />
          <br/>

          <Button
            type="submit"
            disabled={disable}
            onTouchTap={this.delete}
          >
            Delete Upload
          </Button>
        </Form>
      </div>
    );
  }
}


export default Delete;

