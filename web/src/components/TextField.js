import React from 'react';
import { FormGroup, FormControl, ControlLabel } from 'react-bootstrap';


const TextField = ({title, value, update, disabled=false, required=false, error=null}) => {
  let validState = null;
  if (required) { validState = 'warning'; }
  if (error) { validState = 'error'; }

  let errorMessage = null;
  if (required) { errorMessage = 'required'; }
  if (error) { errorMessage = error; }
  return (
    <div>
      <FormGroup
        validationState={validState}
      >
        <ControlLabel>{title}</ControlLabel>
        {' '}
        <FormControl
          type="text"
          value={value}
          placeholder={title}
          onChange={(e) => update(e.target.value)}
          disabled={disabled}
        />
        {' '}
        <FormControl.Feedback />
      </FormGroup>
      {
        errorMessage?
          <span style={{color: 'red'}}> {errorMessage} </span>
          :
          ''
      }
    </div>
  );
}

export default TextField;
