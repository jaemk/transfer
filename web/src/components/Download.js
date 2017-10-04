import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { withRouter } from 'react-router-dom';
import { Button, Form } from 'react-bootstrap';
import axios from 'axios';
import FileSaver from 'file-saver';
import TextField from './TextField';
import PasswordField from './PasswordField';
import ProgressBar from './ProgressBar';
import { logerr } from '../utils/errors';
import { decrypt, bytesFromHex } from '../utils/crypto';


class Download extends Component {
  static propTypes = {
    location: PropTypes.object.isRequired,
  }

  constructor(props) {
    super(props);
    this.state = {
      key: '',
      accessPass: '',
      encryptPass: '',
      errorMessage: '',

      downloadProgress: 0,
      decryptProgress: 0,

      submitted: false,
      inputOk: false,
      required: {},

      responseStatus: null,
    };
    this.download = this.download.bind(this);
    this.catchErr = this.catchErr.bind(this);
  }

  componentWillMount() {
    const search = this.props.location.search;
    const params = new URLSearchParams(search);
    const key = params.get('key');
    if (key) {
      this.setState({key: key});
    }
  }

  catchErr(err) {
    logerr(err);
    this.setState({responseStatus: err.response.status});
  }

  download() {
    let required = {};
    if (this.state.key.length === 0) {
      required.key = true;
    }
    if (this.state.accessPass.length === 0) {
      required.accessPass = true;
    }
    if (this.state.encryptPass.length === 0) {
      required.encryptPass = true;
    }
    let inputOk = true;
    if (Object.keys(required).length > 0) {
      inputOk = false;
    }
    this.setState({required: required, submitted: true, inputOk: inputOk});
    if (!inputOk) {
      return;
    }
    this.setState({
      downloadProgress: 5,
      decryptProgress: 5,
    });

    const decryptedBytesCallback = (bytes, confirmKey) => {
      this.setState({decryptProgress: this.state.decryptProgress + 40});
      window.crypto.subtle.digest('SHA-256', bytes).then(contentHash => {
        this.setState({decryptProgress: 100});
        const hex = Buffer.from(contentHash).toString('hex')
        console.log('decrypted bytes hex', hex)
        const params = {key: confirmKey, hash: hex}
        console.log(`params: ${params}`)
        const headers = {headers: {'content-type': 'application/json'}}
        axios.post('/api/download/confirm', params, headers).then(resp => {
          this.setState({responseStatus: resp.status});
          const blob = new Blob([bytes], {type: 'application/octet-stream'})
          FileSaver.saveAs(blob, resp.data.file_name)
        }).catch(this.catchErr)
      }).catch(this.catchErr)
    }

    const decryptionFailedCallback = e => {
      logerr(e)
      this.setState({responseStatus: 'failed-dec'});
    }

    const params = {key: this.state.key, access_password: Buffer.from(this.state.accessPass).toString('hex')}
    const headers = {headers: {'content-type': 'application/json'}}
    axios.post('/api/download/init', params, headers).then(resp => {
      const nonce = new Uint8Array(bytesFromHex(resp.data.nonce))
      const DLheaders = {
        headers: {'content-type': 'application/json'},
        responseType: 'arraybuffer',
        onDownloadProgress: (event) => {
          this.setState({downloadProgress: (event.loaded / resp.data.size * 100)});
        },
      };

      params.key = resp.data.download_key
      const confirmKey = resp.data.confirm_key
      axios.post('/api/download', params, DLheaders).then(resp => {
        this.setState({downloadProgress: 100});
        const dataBytes = resp.data
        const encryptPassBytes = new TextEncoder().encode(this.state.encryptPass)
        decrypt(dataBytes, nonce, encryptPassBytes,
                (bytes) => decryptedBytesCallback(bytes, confirmKey),
                decryptionFailedCallback)
      }).catch(this.catchErr)
    }).catch(this.catchErr)
  }

  render() {
    const disable = this.state.submitted && this.state.inputOk;
    let message = null;
    switch (this.state.responseStatus) {
      case null:
        break;
      case 'failed-dev':
        message = <div> Decryption Failed </div>;
        break;
      case 200:
        message = <div> Success! </div>;
        break;
      case 400:
        message = <div> Bad Request </div>;
        break;
      case 401:
        message = <div> Invalid credentials </div>;
        break;
      case 404:
        message = <div> Item not found </div>;
        break;
      default:
        message = <div> Something went wrong </div>;
    }
    return (
      <div>
        <Form inline onSubmit={this.download}>
          <TextField
            title="Download Key"
            value={this.state.key}
            disabled={disable}
            update={(v) => this.setState({key: v})}
            required={this.state.required.key}
          />
          <br/>

          <PasswordField
            title="Access"
            disabled={disable}
            update={(v) => this.setState({accessPass: v})}
            required={this.state.required.accessPass}
          />
          <br/>

          <PasswordField
            title="Decrypt"
            disabled={disable}
            update={(v) => this.setState({encryptPass: v})}
            required={this.state.required.encryptPass}
          />
          <br/>

          <Button
            type="submit"
            disabled={disable}
            onTouchTap={this.download}
          >
            Download {'&'} Decrypt
          </Button>
        </Form>

        {
          disable?
            <div>
              <h3> Progress </h3>
              <div>
                {(this.state.downloadProgress > 0)?
                  <ProgressBar
                    title="Downloading File:"
                    active={this.state.downloadProgress < 100}
                    progress={this.state.downloadProgress}
                  />
                    :
                  ''
                }
              </div>
              <div>
                {(this.state.decryptProgress > 0)?
                  <ProgressBar
                    title="Decrypting File:"
                    active={this.state.decryptProgress < 100}
                    progress={this.state.decryptProgress}
                  />
                    :
                  ''
                }
              </div>
            </div>
            :
            ''
        }
        {
          (message === null)?
            ''
            :
            message
        }

      </div>
    )
  }
}


export default withRouter(Download);

