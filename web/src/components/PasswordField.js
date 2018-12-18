import React, { Component } from 'react';
import Form from 'react-bootstrap/lib/Form';


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
    const field1 = this.state.pass;
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
      const valid = val === otherVal;
      const updateVal = valid? val : '';
      this.props.update(updateVal, valid);
    }

    return (
      <div>
        <Form.Group>
          <Form.Group>
            <Form.Label>{this.props.title} Password</Form.Label>
            {' '}
            <Form.Control
              type="password"
              value={this.state.pass}
              placeholder={this.props.title + "Password"}
              onChange={(e) => update(e, 'pass', this.state.confirm)}
              disabled={this.props.disabled}
              isValid={this.state.pass}
              isInvalid={!!this.props.required || (!this.state.pass && this.state.confirm)}
              style={{maxWidth: 350}}
            />
            <Form.Control.Feedback type="invalid">* Required</Form.Control.Feedback>
          </Form.Group>
          {' '}
          {
            this.props.no_confirm? ' ' :
              <div>
                <Form.Control
                  type="password"
                  value={this.state.confirm}
                  placeholder={this.props.title + "Password Confirm"}
                  onChange={(e) => update(e, 'confirm', this.state.pass)}
                  disabled={this.props.disabled}
                  isValid={this.state.pass && this.state.pass === this.state.confirm}
                  isInvalid={(this.state.pass || this.state.confirm) && this.state.pass !== this.state.confirm}
                  style={{maxWidth: 350}}
                />
                <Form.Control.Feedback type="invalid">* Passwords must match</Form.Control.Feedback>
              </div>
          }
        </Form.Group>
      </div>
    );
  }
}

export default PasswordField;

