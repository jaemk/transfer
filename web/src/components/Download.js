import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { withRouter } from 'react-router-dom';
import { Button, Form } from 'react-bootstrap';
import axios from 'axios';
import FileSaver from 'file-saver';
import TextField from './TextField';
import PasswordField from './PasswordField';
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
      submitted: false,
      inputOk: false,
      required: {},
    };
    this.download = this.download.bind(this);
  }

  componentWillMount() {
    const search = this.props.location.search;
    const params = new URLSearchParams(search);
    const key = params.get('key');
    if (key) {
      this.setState({key: key});
    }
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

    const decryptedBytesCallback = (bytes, confirmKey) => {
      window.crypto.subtle.digest('SHA-256', bytes).then(contentHash => {
        const hex = Buffer.from(contentHash).toString('hex')
        console.log('decrypted bytes hex', hex)
        const params = {key: confirmKey, hash: hex}
        console.log(`params: ${params}`)
        const headers = {headers: {'content-type': 'application/json'}}
        axios.post('/api/download/confirm', params, headers).then(resp => {
          const blob = new Blob([bytes], {type: 'application/octet-stream'})
          FileSaver.saveAs(blob, resp.data.file_name)
        }).catch(logerr)
      }).catch(logerr)
    }

    const decryptionFailedCallback = e => {
      logerr(e)
      this.setState({errorMessage: 'Decryption Failed'});
    }

    const params = {key: this.state.key, access_password: Buffer.from(this.state.accessPass).toString('hex')}
    const headers = {headers: {'content-type': 'application/json'}}
    axios.post('/api/download/init', params, headers).then(resp => {
      const nonce = new Uint8Array(bytesFromHex(resp.data.nonce))
      console.log(`nonce: ${nonce}`)
      const DLheaders = {headers: {'content-type': 'application/json'}, responseType: 'arraybuffer'}
      params.key = resp.data.download_key
      const confirmKey = resp.data.confirm_key
      console.log(params)
      axios.post('/api/download', params, DLheaders).then(resp => {
        console.log('post dl')
        console.log(resp)
        const dataBytes = resp.data
        console.log(dataBytes)
        const encryptPassBytes = new TextEncoder().encode(this.state.encryptPass)
        console.log('encrytion pass', encryptPassBytes)
        decrypt(dataBytes, nonce, encryptPassBytes,
                (bytes) => decryptedBytesCallback(bytes, confirmKey),
                decryptionFailedCallback)
      }).catch(logerr)
    }).catch(logerr)
  }

  render() {
    const disable = this.state.submitted && this.state.inputOk;
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
      </div>
    )
  }
}


export default withRouter(Download);

