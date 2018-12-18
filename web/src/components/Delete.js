import React, { Component } from 'react';
import PropTypes from 'prop-types';
import Button from 'react-bootstrap/lib/Button';
import Form from 'react-bootstrap/lib/Form';
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

      responseStatus: null,
    };
    this.delete = this.delete.bind(this);
  }

  componentWillMount() {
    const search = this.props.location.search;
    const params = new URLSearchParams(search);
    let key = params.get('key');
    if (key) {
      [key] = key.split('_');
      this.setState({key: key});
    }
  }

  delete() {
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
      this.setState({
        responseStatus: resp.status,
      });
    }).catch(err => {
      console.error(err);
      this.setState({responseStatus: err.response.status})
    });
  }

  render() {
    const disable = this.state.submitted && this.state.inputOk;
    let message = null;
    switch (this.state.responseStatus) {
      case null:
        break;
      case 200:
        message = <div> Success! </div>;
        break;
      case 404:
        message = <div> Error: Upload not found </div>;
        break;
      case 401:
        message = <div> Error: Invalid credentials </div>;
        break;
      case 400:
        message = <div> Error: Bad request </div>;
        break;
      default:
        message = <div> Something went wrong </div>;
    }

    return (
      <div>
        <Form noValidate onSubmit={this.delete}>
          <TextField
            title="Upload Key"
            value={this.state.key}
            disabled={disable}
            update={(v) => this.setState({key: v})}
            required={this.state.required.key}
          />
          <br/>

          <PasswordField
            title="Deletion"
            disabled={disable}
            update={(v) => this.setState({deletePass: v})}
            required={this.state.required.deletePass}
          />
          <br/>

          <Button
            type="submit"
            disabled={disable}
            onClick={this.delete}
          >
            Delete Upload
          </Button>
        </Form>
        {
          (message === null)?
            ''
            :
            message
        }
      </div>
    );
  }
}


export default Delete;

