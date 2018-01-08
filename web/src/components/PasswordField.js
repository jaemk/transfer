import React, { Component } from 'react';
import { FormGroup, FormControl, ControlLabel } from 'react-bootstrap';


class PasswordField extends Component {
  /**
   * props: title, update, disabled, required
   */

  constructor(props) {
    super(props);
    this.state = {
      pass: '',
      confirm: '',
    };

    this.validate = this.validate.bind(this);
  }

  validate() {
    const field1 = this.state.access;
    if (field1 && this.props.no_confirm) { return 'success'; }

    const field2 = this.state.confirm;
    if (field1 && field2) {
      return (field1 === field2)? 'success' : 'error';
    }
    return null;
  }

  render() {
    const update = (e, field, otherVal) => {
      const val = e.target.value;
      let params = {};
      params[field] = val;
      this.setState(params);

      if (this.props.no_confirm) {
        this.props.update(val);
        return
      }
      const updateVal = (val === otherVal)? val : '';
      this.props.update(updateVal);
    }

    return (
      <div>
        <FormGroup
          validationState={this.props.required? 'warning' : this.validate()}
        >
          <ControlLabel>{this.props.title} Password</ControlLabel>
          {' '}
          <FormControl
            type="password"
            value={this.state.pass}
            placeholder={this.props.title + "Password"}
            onChange={(e) => update(e, 'pass', this.state.confirm)}
            disabled={this.props.disabled}
          />
          {' '}
          {
            this.props.no_confirm? ' ' :
              <FormControl
                type="password"
                value={this.state.confirm}
                placeholder={this.props.title + "Password Confirm"}
                onChange={(e) => update(e, 'confirm', this.state.pass)}
                disabled={this.props.disabled}
              />
          }
          <FormControl.Feedback />
        </FormGroup>
        {
          this.props.required?
            <span style={{color: 'red'}}> required </span>
            :
            ''
        }
      </div>
    );
  }
}

export default PasswordField;

