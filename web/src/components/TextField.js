import React from 'react';
import Form from 'react-bootstrap/lib/Form';
import FormControl from 'react-bootstrap/lib/FormControl';


const TextField = ({title, value, update, type="text", disabled=false, required=false, error=null}) => {
  let errorMessage = null;
  if (required) { errorMessage = '* Required'; }
  if (error) { errorMessage = error; }
  return (
    <div>
      <Form.Group>
        <Form.Label>{title}</Form.Label>
        {' '}
        <Form.Control
          type={type}
          value={value}
          placeholder={title}
          onChange={(e) => update(e.target.value)}
          disabled={disabled}
          isInvalid={required || error}
          style={{maxWidth: 350}}
        />
        {' '}
        <FormControl.Feedback type="invalid"> { errorMessage } </FormControl.Feedback>
      </Form.Group>
    </div>
  );
}
export default TextField;
