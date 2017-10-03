import React, { Component } from 'react';
import { NavLink } from 'react-router-dom';
import { Button, Form } from 'react-bootstrap';
import axios from 'axios';
import PasswordField from './PasswordField';
import TextField from './TextField';
import FileField from './FileField';
import { logerr } from '../utils/errors';
import { randomBytes, encrypt } from '../utils/crypto';


class Upload extends Component {
  constructor(props) {
    super(props);
    this.state = {
      accessPass: '',
      encryptPass: '',
      deletePass: '',
      deletePassConfirm: '',
      downloadLimit: '',
      lifespan: '',

      downloadUrl: null,
      submitted: false,
      inputOk: false,
      required: {},
      errors: {},
    };
    this.submit = this.submit.bind(this);
  }


  submit(e) {
    console.log(this.state);

    let file = document.getElementById('file').files[0]

    let required = {};
    if (!file) {
      required.file = true;
    }
    if (this.state.accessPass.length === 0) {
      required.accessPass = true;
    }
    if (this.state.encryptPass.length === 0) {
      required.encryptPass = true;
    }

    let errors = {};
    const downloadLimit = (this.state.downloadLimit === '')? null : parseInt(this.state.downloadLimit, 10);
    if (downloadLimit !== null && (isNaN(downloadLimit) || downloadLimit < 1 || downloadLimit > 999)) {
      console.log(downloadLimit);
      errors.downloadLimit = 'expected an integer, 1 - 999';
    }
    const lifespan = (this.state.lifespan === '')? null : parseInt(this.state.lifespan, 10);
    if (lifespan !== null && (isNaN(lifespan) || lifespan < 1 || lifespan > 604800)) {
      errors.lifespan = 'expected an integer, 1 - 604800';
    }

    let inputOk = true;
    if (Object.keys(required).length > 0 || Object.keys(errors).length > 0) {
      inputOk = false;
    }
    this.setState({required: required, errors: errors, submitted: true, inputOk: inputOk});
    if (!inputOk) {
      return;
    }

    const nonce = randomBytes(12)
    const encryptPassBytes = new TextEncoder().encode(this.state.encryptPass)
    const accessPassBytes = new TextEncoder().encode(this.state.accessPass)
    const nonceHex = Buffer.from(nonce).toString('hex')
    const accessPassHex = Buffer.from(accessPassBytes).toString('hex')
    let deletePassHex = null
    if (this.state.deletePass.length > 0) {
      const deletePassBytes = new TextEncoder().encode(this.state.deletePass)
      deletePassHex = Buffer.from(deletePassBytes).toString('hex')
    }

    const encryptUploadData = (data, params, headers) => {
      const encryptedBytesCallback = (bytes) => {
        params.size = bytes.length
        axios.post('/api/upload/init', params, headers).then(resp => {
          const key = resp.data.key
          console.log(`got key ${key}`)
          axios.post(`/api/upload?key=${key}`, bytes, {headers: {'content-type': 'application/octet-stream'}})
            .then(resp => {
              console.log(resp.data)
              console.log(`key: ${key}`)
              this.setState({downloadUrl: `/download?key=${key}`});
            }).catch(logerr)
        }).catch(logerr)
      }
      encrypt(data, nonce, encryptPassBytes, encryptedBytesCallback)
    }

    let reader = new FileReader()
    reader.onload = (event) => {
      if (reader.readyState !== 2) {
        console.log(`read ${event.loaded} bytes`)
        return
      }
      const data = reader.result
      console.log(`loaded ${data.byteLength} bytes`)
      window.crypto.subtle.digest('SHA-256', data).then(contentHash => {
        const contentHashHex = Buffer.from(contentHash).toString('Hex')
        console.log('content hash', contentHashHex)
        const params = {
          nonce: nonceHex,
          file_name: file.name,
          content_hash: contentHashHex,
          access_password: accessPassHex,
          deletion_password: deletePassHex,
          download_limit: downloadLimit,
          lifespan: lifespan,
        }
        const headers = {headers: {'content-type': 'application/json'}}
        encryptUploadData(data, params, headers)
      }).catch(logerr)
    }
    reader.readAsArrayBuffer(file)
  }

  render() {
    const disable = this.state.submitted && this.state.inputOk;
    return (
      <div>
        {
          this.state.downloadUrl?
            <NavLink to={this.state.downloadUrl}>
              Download here: {`http://${window.location.host}${this.state.downloadUrl}`}
            </NavLink>
            :
            ''
        }
        <Form inline onSubmit={this.submit}>
          <FileField
            title="Upload file"
            domId="file"
            disabled={disable}
            required={this.state.required.file}
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
            title="Encrypt"
            disabled={disable}
            update={(v) => this.setState({encryptPass: v})}
            required={this.state.required.encryptPass}
          />
          <br/>

          <PasswordField
            title="Delete"
            disabled={disable}
            update={(v) => this.setState({deletePass: v})}
          />
          <br/>

          <TextField
            title="Download Limit"
            value={this.state.downloadLimit}
            disabled={disable}
            update={(v) => this.setState({downloadLimit: v})}
            required={false}
            error={this.state.errors.downloadLimit}
          />
          <br/>

          <TextField
            title="Lifespan (seconds)"
            value={this.state.lifespan}
            disabled={disable}
            update={(v) => this.setState({lifespan: v})}
            required={false}
            error={this.state.errors.lifespan}
          />
          <br/>

          <Button
            type="submit"
            disabled={disable}
            onTouchTap={this.submit}
          >
            Encrypt {'&'} Upload
          </Button>
        </Form>
      </div>
    )
  }
}

export default Upload;

