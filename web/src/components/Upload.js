import React, { Component } from 'react';
import { NavLink } from 'react-router-dom';
import { Button, Form } from 'react-bootstrap';
import axios from 'axios';
import PasswordField from './PasswordField';
import TextField from './TextField';
import FileField from './FileField';
import ProgressBar from './ProgressBar';
import { logerr } from '../utils/errors';
import { randomBytes, encrypt } from '../utils/crypto';


class Upload extends Component {
  constructor(props) {
    super(props);
    this.state = {
      accessPass: '',
      accessValid: false,
      encryptPass: '',
      encryptValid: false,
      deletePass: '',
      deleteValid: true,
      downloadLimit: '',
      lifespan: '',

      loadProgress: 0,
      encryptProgress: 0,
      uploadProgress: 0,

      downloadUrl: null,
      submitted: false,
      inputOk: false,
      required: {},
      errors: {},

      responseStatus: null,
    };
    this.submit = this.submit.bind(this);
    this.catchErr = this.catchErr.bind(this);
  }

  catchErr(err) {
    logerr(err);
    this.setState({responseStatus: err.response.status});
  }


  submit(e) {
    let file = document.getElementById('file').files[0]

    let required = {};
    if (!file) {
      required.file = true;
    }
    if (this.state.accessPass.length === 0 || !this.state.accessValid) {
      required.accessPass = true;
    }
    if (this.state.encryptPass.length === 0 || !this.state.encryptValid) {
      required.encryptPass = true;
    }
    if (!this.state.deleteValid) {
      required.deletePass = true;
    }

    let errors = {};
    const downloadLimit = (this.state.downloadLimit === '')? null : parseInt(this.state.downloadLimit, 10);
    if (downloadLimit !== null && (isNaN(downloadLimit) || downloadLimit < 1 || downloadLimit > 999)) {
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

    this.setState({
      loadProgress: 5,
      encryptProgress: 5,
      uploadProgress: 5,
    });

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

    const encryptUploadData = (data, file_name, params, headers) => {
      const encryptedBytesCallback = (bytes) => {
        this.setState({encryptProgress: 100});
        params.size = bytes.length
        axios.post('/api/upload/init', params, headers).then(resp => {
          const file_name_b64 = btoa(file_name)
          const fileNameContainingKey = resp.data.key + '_' + file_name_b64
          const key = resp.data.key
          this.setState({key: fileNameContainingKey});
          const config = {
            headers: {'content-type': 'application/octet-stream'},
            onUploadProgress: (event) => this.setState({uploadProgress: (event.loaded / event.total * 100)}),
          };
          axios.post(`/api/upload?key=${key}`, bytes, config)
            .then(resp => {
              this.setState({
                downloadUrl: `/download?key=${fileNameContainingKey}`,
                uploadProgress: 100,
                responseStatus: resp.status,
              });
            }).catch(this.catchErr)
        }).catch(this.catchErr)
      }
      encrypt(data, nonce, encryptPassBytes, encryptedBytesCallback)
    }

    let reader = new FileReader()
    reader.onload = (event) => {
      if (reader.readyState !== 2) {
        return
      }
      console.log(`read ${event.loaded} bytes`)
      this.setState({loadProgress: 100});

      const data = reader.result
      const fileNameBytes = new TextEncoder().encode(file.name)
      console.log(`loaded ${data.byteLength} bytes`)
      window.crypto.subtle.digest('SHA-256', data).then(contentHash => {
        window.crypto.subtle.digest('SHA-256', fileNameBytes).then(fileNameHash => {
          this.setState({encryptProgress: this.state.encryptProgress + 60});
          const contentHashHex = Buffer.from(contentHash).toString('Hex')
          const fileNameHashHex = Buffer.from(fileNameHash).toString('Hex')
          console.log('content hash', contentHashHex)
          console.log('file_name hash', fileNameHashHex)
          const params = {
            nonce: nonceHex,
            file_name_hash: fileNameHashHex,
            content_hash: contentHashHex,
            access_password: accessPassHex,
            deletion_password: deletePassHex,
            download_limit: downloadLimit,
            lifespan: lifespan,
          }
          const headers = {headers: {'content-type': 'application/json'}}
          encryptUploadData(data, file.name, params, headers)
        }).catch(this.catchErr)
      }).catch(this.catchErr)
    }
    reader.onprogress = (event) => {
      console.log(`in progress: read ${event.loaded} bytes`);
      const progress = event.loaded / event.total * 100;
      this.setState({loadProgress: progress});
    }
    reader.readAsArrayBuffer(file)
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
      case 413:
        message = <div> Upload too large </div>;
        break;
      default:
        message = <div> Something went wrong </div>;
    }
    return (
      <div>
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
            update={(v, valid) => this.setState({accessPass: v, accessValid: valid})}
            required={this.state.required.accessPass}
          />
          <br/>

          <PasswordField
            title="Encrypt"
            disabled={disable}
            update={(v, valid) => this.setState({encryptPass: v, encryptValid: valid})}
            required={this.state.required.encryptPass}
          />
          <br/>

          <PasswordField
            title="Delete"
            disabled={disable}
            update={(v, valid) => this.setState({deletePass: v, deleteValid: valid})}
            required={this.state.required.deletePass}
          />
          <br/>

          <TextField
            title="Download Limit"
            type="number"
            value={this.state.downloadLimit}
            disabled={disable}
            update={(v) => this.setState({downloadLimit: v})}
            required={false}
            error={this.state.errors.downloadLimit}
          />
          <br/>

          <TextField
            title="Lifespan (seconds)"
            type="number"
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
            onClick={this.submit}
          >
            Encrypt {'&'} Upload
          </Button>
        </Form>

        <br/>

        {
          disable?
            <div>
              <h3> Progress </h3>
              <div>
                {(this.state.loadProgress > 0)?
                  <ProgressBar
                    title="Loading File:"
                    active={this.state.loadProgress < 100}
                    progress={this.state.loadProgress}
                  />
                    :
                  ''
                }
              </div>
              <div>
                {(this.state.encryptProgress > 0)?
                  <ProgressBar
                    title="Encrypting File:"
                    active={this.state.encryptProgress < 100}
                    progress={this.state.encryptProgress}
                  />
                    :
                  ''
                }
              </div>
              <div>
                {(this.state.uploadProgress > 0)?
                  <ProgressBar
                    title="Uploading File:"
                    active={this.state.uploadProgress < 100}
                    progress={this.state.uploadProgress}
                  />
                    :
                  ''
                }
              </div>

              {
                this.state.key?
                  <div>
                    Upload/Download Key: <code> {this.state.key} </code>
                  </div>
                  :
                  ''
              }

              {
                this.state.downloadUrl?
                  <div>
                    Download Link:
                    <code>
                      <NavLink to={this.state.downloadUrl}>
                        {`http://${window.location.host}${this.state.downloadUrl}`}
                      </NavLink>
                    </code>
                  </div>
                  :
                  ''
              }
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

export default Upload;

