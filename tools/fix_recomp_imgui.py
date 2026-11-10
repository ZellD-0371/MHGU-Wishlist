"""Narrow, reversible fix for MHGU Recomp's borrowed ImDrawList vertex cursor.
Matches three complete adjacent descriptors and verifies a read-only PE data section.
No game code, ABI, saves, DLLs, or unrelated executable bytes are changed.
"""
import argparse,hashlib,json,os,pathlib,struct

def digest(b):return hashlib.sha256(b).hexdigest()
def record(g,h,kind,typ):return struct.pack('<IIIBxH',g,h,0,kind,typ)
OLD=record(44,56,3,8)+record(48,64,3,10)+record(52,72,4,0)
NEW=record(44,56,3,8)+record(48,64,4,0)+record(52,72,4,0)
def inspect(data):
 if data[:2]!=b'MZ':raise ValueError('Expected a Windows PE executable')
 pe=struct.unpack_from('<I',data,0x3c)[0]
 if data[pe:pe+4]!=b'PE\0\0':raise ValueError('Invalid PE signature')
 sections=struct.unpack_from('<H',data,pe+6)[0];opt=struct.unpack_from('<H',data,pe+20)[0]
 if data.count(OLD)==0 and data.count(NEW)==1:return None
 if data.count(OLD)!=1 or data.count(NEW)!=0:raise ValueError('Unsupported or ambiguous cursor table; no changes made')
 offset=data.index(OLD)
 for i in range(sections):
  pos=pe+24+opt+i*40;size,start=struct.unpack_from('<II',data,pos+16);flags=struct.unpack_from('<I',data,pos+36)[0]
  if start<=offset and offset+len(OLD)<=start+size:
   if not flags&0x40000000 or flags&0x20000020:raise ValueError('Cursor table is not in a read-only data section')
   return offset
 raise ValueError('Cursor table is outside PE sections')

def main():
 parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('exe',type=pathlib.Path);parser.add_argument('--apply',action='store_true');parser.add_argument('--restore',action='store_true');parser.add_argument('--backup-dir',type=pathlib.Path,required=True);args=parser.parse_args()
 target=args.exe.resolve();folder=args.backup_dir.resolve();data=target.read_bytes();manifest=folder/'imgui-cursor-fix.json';backup=folder/'mhgurecomp.before-imgui-cursor-fix.exe'
 if args.restore:
  m=json.loads(manifest.read_text());original=backup.read_bytes()
  if digest(data)!=m['patched_sha256'] or digest(original)!=m['original_sha256']:raise ValueError('File differs from the recorded versions; restore refused')
  temp=target.with_name(target.name+'.wishlist-tmp');temp.write_bytes(original);os.replace(temp,target);print('Restored original executable');return
 offset=inspect(data)
 if offset is None:print('Already fixed');return
 print('Verified unique read-only ImDrawList cursor descriptor at',hex(offset+16))
 if not args.apply:print('Inspection only; no file changed');return
 if backup.exists() or manifest.exists():raise ValueError('Backup already exists; refusing to overwrite it')
 fixed=data[:offset]+NEW+data[offset+len(OLD):]
 changes=[i for i,(a,b) in enumerate(zip(data,fixed)) if a!=b]
 if changes!=[offset+28,offset+30]:raise ValueError('Unexpected binary changes')
 assert inspect(fixed) is None
 folder.mkdir(parents=True,exist_ok=True);backup.write_bytes(data)
 info={'target':str(target),'descriptor_file_offset':offset+16,'changed_offsets':changes,'original_sha256':digest(data),'patched_sha256':digest(fixed),'reason':'_VtxWritePtr is borrowed buffer data, not an ImDrawVert object'}
 manifest.write_text(json.dumps(info,indent=2),encoding='utf-8')
 temp=target.with_name(target.name+'.wishlist-tmp');temp.write_bytes(fixed);os.replace(temp,target)
 if target.read_bytes()!=fixed:raise ValueError('Read-back verification failed')
 print(json.dumps(info,indent=2));print('Applied and verified; original backup preserved')
if __name__=='__main__':main()
