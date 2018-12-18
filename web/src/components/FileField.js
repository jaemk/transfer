import React from 'react';
import Form from 'react-bootstrap/lib/Form';


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
    <Form.Group>
      <Form.Label>{title}</Form.Label>
      {' '}
      <Form.Control
        id={domId}
        type="file"
        disabled={disabled}
        isInvalid={!!required}
      />
      <Form.Control.Feedback type="invalid">* Required</Form.Control.Feedback>
    </Form.Group>
  );
}

export default FileField;

