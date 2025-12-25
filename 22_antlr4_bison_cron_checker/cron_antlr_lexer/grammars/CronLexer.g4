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

DOW_NAMEa 
    : 'SUN' | 'MON' | 'TUE' | 'WED' | 'THU' | 'FRI' | 'SAT'
    ;

INT : [0-9]+ ;

URL
    : ('http' | 'https' | 'ftp' | 'ssh') '://' ~[ \t\r\n#]+
    ;

// PATH tokens: quoted paths, tilde-expanded paths, absolute or relative paths
PATH
    : QUOTED_PATH
    | TILDE_PATH
    | ABS_PATH
    | REL_PATH
    ;

fragment QUOTED_PATH
    : '"' ( '\\' . | ~[\\"\r\n] )* '"'
    | '\'' ( '\\' . | ~[\\'\r\n] )* '\''
    ;

fragment TILDE_PATH
    : '~' ~[ \t\r\n]*
    ;

fragment ABS_PATH
    : '/' (~[ \t\r\n/])+ ('/' ~[ \t\r\n/]*)*
    ;

fragment REL_PATH
    : (~[ \t\r\n/])+ ('/' ~[ \t\r\n/]*)+
    ;

CLI_OPTION
    : '--' [a-zA-Z0-9][a-zA-Z0-9_-]*
    | '-' [a-zA-Z][a-zA-Z0-9_-]*
    ;

// PROGRAM is intentionally last so it acts as a fallback token for any
// remaining command text that wasn't captured by other rules.
PROGRAM
    : (~[ \t\r\n#/])+
    ;
