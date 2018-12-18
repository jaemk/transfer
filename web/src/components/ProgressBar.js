import React from 'react';
import BSProgressBar from 'react-bootstrap/lib/ProgressBar';


const ProgressBar = ({title, active, progress}) => {
  return (
    <div>
      <h4> {title} </h4>
      <BSProgressBar
        striped
        animated={active}
        variant={(progress >= 100)? 'success' : 'warning'}
        now={progress}
      />
    </div>
  );
};


export default ProgressBar;

