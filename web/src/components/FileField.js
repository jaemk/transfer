import React from 'react';
import { FormGroup, FormControl, ControlLabel } from 'react-bootstrap';


/**
 * FileField
 *
 * @param title {string} - form field title
 * @param domId {string} - id to assign to element
 * @param disabled {boolean} - disable field
 * @param required {boolean} - mark with a warning that field is required
 */
const FileField = ({title, domId, disabled, required}) => {
  return (
    <FormGroup
      validationState={required? 'warning' : null}
    >
      <ControlLabel>{title}</ControlLabel>
      {' '}
      <FormControl
        id={domId}
        type="file"
        disabled={disabled}
      />
      {
        required?
          <span style={{color: 'red'}}> required </span>
          :
          ''
      }
    </FormGroup>
  );
}

export default FileField;

