lexer grammar CronLexer;

WS : [ \t\r\n]+ -> skip ;

CRON_COMMENT : '#' ~[\r\n]* -> skip ;

STAR        : '*';
SLASH       : '/';
COMMA       : ',';
DASH        : '-';

MONTH_NAME
    : 'JAN' | 'FEB' | 'MAR' | 'APR' | 'MAY' | 'JUN'
    | 'JUL' | 'AUG' | 'SEP' | 'OCT' | 'NOV' | 'DEC'
    ;

DOW_NAME
    : 'SUN' | 'MON' | 'TUE' | 'WED' | 'THU' | 'FRI' | 'SAT'
    ;

INT : [0-9]+ ;

// PATH tokens: quoted paths, tilde-expanded paths, absolute or relative paths
PATH
    : QUOTED_PATH
    | TILDE_PATH
    | ABS_OR_REL_PATH
    ;

fragment QUOTED_PATH
    : '"' ( '\\' . | ~[\\"\r\n] )* '"'
    | '\'' ( '\\' . | ~[\\'\r\n] )* '\''
    ;

fragment TILDE_PATH
    : '~' ~[ \t\r\n]*
    ;

fragment ABS_OR_REL_PATH
    : (~[ \t\r\n/])+ ('/' ~[ \t\r\n/]*)+
    ;

COMMAND : ~[\r\n]* ;