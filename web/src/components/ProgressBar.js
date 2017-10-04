import React from 'react';
import { ProgressBar as BSProgressBar } from 'react-bootstrap';


const ProgressBar = ({title, active, progress}) => {
  return (
    <div>
      <h4> {title} </h4>
      <BSProgressBar
        striped
        active={active}
        bsStyle={(progress >= 100)? 'success' : 'warning'}
        now={progress}
      />
    </div>
  );
};


export default ProgressBar;

