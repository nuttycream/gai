use crate::consts::COMMIT_CONVENTION;

pub fn build_prompt(
    use_convention: bool,
    sys_prompt: &str,
    rules: &str,
    use_hunk: bool,
) -> String {
    let mut prompt = String::new();

    prompt.push_str(sys_prompt);
    prompt.push('\n');

    prompt.push_str(rules);
    prompt.push('\n');

    if use_convention {
        prompt.push_str(COMMIT_CONVENTION);
    }

    prompt.push('\n');

    if use_hunk {
        prompt.push_str(
            "fill hunk_headers with specific hunks from each file",
        );
    } else {
        prompt.push_str(
            "fill out path files, leave hunk_headers empty",
        );
    }

    prompt
}

pub const LOGO: &str = r#"
                           :o#@@@#s'                                
                          ?Q@@@@@@@g,                               
                         `d@@@@@@@QQs                V#############g
                          tQQQQQQQQQs                6QQQ@@@@@@@Q@@Q
                          `\dQQQQQQQQv`              `````o@@@#'````
                 |UD8RDe`   ,#QQQOBQQQm'                  u@@@N`    
       `,;;^!;_''QQW&&8m`   ,g###i>D###d;                 f@@@g`    
    ,zOQ@@@QQQQQgQa`        'Rggg/ ,GggNWi`               ]@@@g`    
  'e@@@QPs\??/FKQQ#2'       'H88R\  `vD&Wgo'              }@@@&`    
 ,Q@@@s`        ,g##H"      'KHdd|    "e%D8d?`            tQ@@8`    
 F@@@8`          jNNgt      '9KKp?     .sd%DRf'           tQ@@8`    
 S@@@O           sNgWj      'k99U=       ^9%%Rh/|^'       zQQ@R`    
 =@@@Q"          PNg&|      'PkkPr        :p%R8&ggg\`     zQQ@R`    
 `F@@@g?.      ,tNgWF`      'hkkPr        |H%R8&gNN#\     zQQ@R`    
   ^OQ@QQdPeeP%##NP^`       'k9U6=        7%DR&WgN##z     zQQ@R`    
    '7#Q#&N#N8qal_         '\pKpp}`       'V8&WgNN#P'     tQ@@8`    
  ;KQQ7~` ```             ?p%HHdddh;       `;seUPj?`      }Q@@8`    
 ~Q@@2                   :R&88RRDDDp'                     }@@@&`    
 'B@@@QHkXGmeeeVofs\;'   'KNgggggggG`                !///iO@@@Qs777i
  '}Q@@@@@@@QQQQQQQQQBX;  ,oN####Nf'                 6Q@@@@@@@@@@@@@
`rm#Qj>^^!!!;;;;^=7GQQQQ|   '!??^,                   ;\|\\\\///////\
e@@Q:               ^QQQO`                                          
@@@Q.               :Q@QH`                                          
#@@@O:            ,iN@@Q?                                           
:O@@@@QD9hXXXPkqDQ@@@@O;                                            
  _iVH#Q@@@@@@@@QWqS\,
"#;
