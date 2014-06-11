// Copyright 2014: Ty Overby

#[macro_export]
macro_rules! match_any(                                                                                                
    ($val:expr match  $(if $typ:ty { $($patn: pat => $exp: expr),+  }),+ else { $other: expr}) => (                    
        {                                                                                                              
            $(                                                                                                         
                if $val.is::<$typ>() {                                                                                 
                    match $val.as_ref::<$typ>().unwrap() {                                                             
                        $($patn => $exp),*                                                                             
                    }                                                                                                  
                } else                                                                                                 
            )*                                                                                                         
            {                                                                                                          
                $other                                                                                                 
            }                                                                                                          
        }                                                                                                              
    )                                                                                                                  
)                                                                                                                      
